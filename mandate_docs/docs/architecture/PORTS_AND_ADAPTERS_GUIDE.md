# Ports &amp; Adapters Architecture (Hexagonal Architecture)

*A practical reference guide for developers*

---

## 1. What is Ports &amp; Adapters?

Ports &amp; Adapters — coined by Alistair Cockburn as **Hexagonal Architecture** — is an architectural pattern with one central idea:

&gt; **Your business logic should not know or care how the outside world talks to it, or how it talks to the outside world.**

The application core (domain) sits in the middle. Everything external — databases, web frameworks, message queues, file systems, third-party APIs, CLIs, test harnesses — connects to the core only through **ports** (abstract interfaces the core defines) and **adapters** (concrete implementations that satisfy those interfaces).

### Why it matters

- **Testability.** The domain can be tested with in-memory fakes — no database, no HTTP server, no mocks of vendor SDKs. Tests run in milliseconds.
- **Flexibility.** Swap Postgres for DynamoDB, REST for gRPC, or add an MCP tool interface next to your CLI — without touching business rules.
- **Maintainability.** Framework churn, SDK upgrades, and vendor migrations are contained to the adapter layer. The core stays stable.
- **Deferred decisions.** You can build and validate business logic before choosing infrastructure. The database becomes a detail, not a foundation.

### The hexagon

The hexagon shape is not literal — it simply signals "many sides, many possible connections." Left side: things that *drive* the application. Right side: things the application *drives*.

```
        DRIVING (primary)                        DRIVEN (secondary)
     "things that call you"                   "things you call out to"

  ┌──────────────┐                                  ┌──────────────┐
  │  HTTP        │──┐                            ┌──│  Postgres    │
  │  Controller  │  │                            │  │  Adapter     │
  └──────────────┘  │    ┌──────────────────┐    │  └──────────────┘
                    ├───▶│                  │────┤
  ┌──────────────┐  │    │   APPLICATION    │    │  ┌──────────────┐
  │  CLI Handler │──┤    │      CORE        │    ├──│  SMTP Email  │
  └──────────────┘  │    │                  │    │  │  Adapter     │
                    │    │  domain models   │    │  └──────────────┘
  ┌──────────────┐  │    │  business rules  │    │
  │  MCP Tool /  │──┤    │  use cases       │    │  ┌──────────────┐
  │  Test Suite  │  │    │                  │    ├──│  S3 Storage  │
  └──────────────┘  │    └──────────────────┘    │  │  Adapter     │
                    │      ▲            ▲        │  └──────────────┘
                 driving   │            │  driven│
                 PORTS ────┘            └── PORTS
            (interfaces the core       (interfaces the core
             exposes: use cases)        depends on: repositories,
                                        gateways, notifiers)
```

Both sides cross the boundary through ports. Adapters live *outside* the hexagon; ports and domain live *inside*.

---

## 2. Core Principles

### Dependency inversion

The domain **never imports** an implementation. It defines an interface ("I need something that can save orders") and infrastructure code implements it. The classic layered mistake — `OrderService` imports `PostgresClient` — is inverted: `PostgresOrderRepository` imports `OrderRepository` (the port), which the domain owns.

**The domain owns the ports.** This is the crux. The interface lives in the core, written in the core's vocabulary, shaped by the core's needs — not by what the database happens to offer.

### Ports as contracts

A port is an abstract interface: a promise about *what* is needed or offered, with no statement about *how*. It is the entire API surface between the core and one category of external concern.

### Adapters as implementations

An adapter is the concrete wiring: it implements a port using a specific technology (a SQL driver, an HTTP client, an SDK) or translates external input (an HTTP request, a CLI arg) into calls on the core.

### Clear boundaries

Every crossing between "inside" and "outside" goes through a port. No shortcuts, no "just this once" direct imports. The boundary is only as strong as its weakest crossing.

### Why business logic must be isolated

Business rules are the most valuable, longest-lived code in your system. Frameworks and databases are the most volatile. Coupling the durable to the volatile means every infrastructure change risks your rules — and every rule change requires spinning up infrastructure to test. Isolation breaks that coupling in both directions.

---

## 3. Anatomy of the Pattern

### Domain

Business models, invariants, and workflows (use cases / application services). Pure logic — no I/O, no framework imports, no SDK types.

```ts
// domain/order.ts — a model with behavior and invariants
class Order {
  addItem(item: LineItem): void { /* enforce business rules */ }
  total(): Money { /* pure calculation */ }
}

// domain/place-order.ts — a use case orchestrating via ports
class PlaceOrder {
  constructor(
    private orders: OrderRepository,     // driven port
    private payments: PaymentGateway,    // driven port
    private notifier: OrderNotifier,     // driven port
  ) {}

  async execute(cmd: PlaceOrderCommand): Promise&lt;OrderConfirmation&gt; {
    // pure orchestration: validate, charge, persist, notify
  }
}
```

### Ports

Interfaces owned by the domain, in domain vocabulary:

```ts
// ports/order-repository.ts
interface OrderRepository {
  findById(id: OrderId): Promise&lt;Order | null&gt;;
  save(order: Order): Promise&lt;void&gt;;
}

// ports/payment-gateway.ts
interface PaymentGateway {
  charge(amount: Money, method: PaymentMethod): Promise&lt;PaymentResult&gt;;
}
```

### Adapters

Concrete implementations, one per technology:

```ts
// adapters/persistence/postgres-order-repository.ts
class PostgresOrderRepository implements OrderRepository {
  constructor(private pool: Pool) {}
  async save(order: Order) { /* SQL lives here, and only here */ }
}

// adapters/http/order-controller.ts  (driving adapter)
class OrderController {
  constructor(private placeOrder: PlaceOrder) {}
  async post(req: Request): Promise&lt;Response&gt; {
    const cmd = toCommand(req.body);           // translate inward
    const result = await this.placeOrder.execute(cmd);
    return toResponse(result);                  // translate outward
  }
}
```

### Anti-corruption layers

When an external system's model is messy or foreign (a legacy ERP, a vendor API with awkward semantics), the adapter should include a **translation layer** that converts the external model into clean domain types at the boundary. Never let vendor DTOs, ORM entities, or wire formats flow into the core. The adapter absorbs the ugliness so the domain doesn't have to.

---

## 4. Ports (Abstract Interfaces)

### What makes a good port

- **Expressed in domain language.** `OrderRepository.save(order)`, not `Database.executeQuery(sql)`.
- **Technology-agnostic.** Nothing in the signature hints at SQL, HTTP, or a vendor. If you renamed the port's implementation from Postgres to DynamoDB, the interface shouldn't need to change.
- **Shaped by the consumer, not the provider.** Define the methods the use case actually needs — no more.

### Single, focused responsibility

One port per *capability*, not one giant `IInfrastructure` interface. `OrderRepository`, `PaymentGateway`, and `EmailNotifier` are three ports, even if today they're all "external stuff." Small ports are easier to fake, easier to swap, and harder to abuse.

### Naming conventions

| Category | Convention | Examples |
|---|---|---|
| Persistence | `&lt;Aggregate&gt;Repository` | `OrderRepository`, `UserRepository` |
| External services | `&lt;Capability&gt;Gateway` / `&lt;Capability&gt;Provider` | `PaymentGateway`, `GeocodingProvider` |
| Outbound messaging | `&lt;Event&gt;Publisher`, `&lt;Thing&gt;Notifier` | `OrderEventPublisher`, `EmailNotifier` |
| Cross-cutting | `Clock`, `IdGenerator`, `Logger` | — |
| Driving (inbound) | `&lt;UseCase&gt;` / `&lt;Action&gt;&lt;Noun&gt;` | `PlaceOrder`, `CancelSubscription` |

### Common port categories

- **Persistence:** repositories, unit-of-work.
- **External services:** payment, email/SMS, search, geocoding, LLM providers.
- **Formatting/presentation:** report renderers, exporters, serializers — when output format varies.
- **Environment:** clock, random/ID generation, configuration reading — anything nondeterministic.

### How ports enable swappability

Because the domain compiles against the interface only, any implementation satisfying the contract is a drop-in: `PostgresOrderRepository` in production, `InMemoryOrderRepository` in tests, `CachedOrderRepository` wrapping either. Swapping is a wiring change, not a refactor.

---

## 5. Adapters (Implementations)

### Driving adapters (primary)

Translate external *input* into calls on the core: HTTP controllers, CLI handlers, MCP tool handlers, message consumers, scheduled jobs, GUI event handlers — and your test suite (tests are just another driving adapter). Their job: parse/validate transport-level input, build a command, invoke a use case, translate the result back to the transport format. **No business decisions.**

### Driven adapters (secondary)

Implement ports the core *calls out to*: SQL/NoSQL repositories, file-system storage, HTTP clients for third-party APIs, SMTP senders, message publishers. Their job: fulfill the contract using a specific technology, and translate infrastructure errors into port-level errors (`OrderNotFound`, not `PgError 23505`).

### Adapter isolation

Adapters must not contain business rules ("orders over $500 need approval" belongs in the domain, never in a controller) and must not leak their internals (no ORM entities or `ResultSet`s returned through a port). An adapter should be *boring*: translation, wiring, error mapping.

### Plugin architecture

Because adapters are interchangeable behind ports, third parties (or future you) can supply new ones without modifying the core — see Section 10.

### Testing adapters in isolation

- **Driven adapters:** integration tests against the real technology (Testcontainers for a database, a sandbox account for a payment API). Verify the adapter honors the port contract.
- **Driving adapters:** thin tests confirming translation — request in, correct command out; result in, correct response out — with the use case faked.
- **Contract tests:** run one shared test suite against *every* implementation of a port (in-memory fake included) so all adapters behave identically.

---

## 6. Dependency Flow

### Arrows point inward

At compile time, all source dependencies point toward the domain:

```
  adapters ───▶ ports ◀─── (owned by) ─── domain
     │                                      ▲
     └── imports domain types ──────────────┘

  NEVER: domain ───▶ adapters
```

At *runtime*, control flows outward (the use case calls the repository), but the *source-code dependency* still points inward — that's the inversion.

### Dependency injection

Ports are wired to adapters via constructor injection. The domain receives its collaborators; it never constructs them:

```ts
// GOOD: dependencies injected
class PlaceOrder {
  constructor(private orders: OrderRepository) {}
}

// BAD: domain constructs infrastructure
class PlaceOrder {
  private orders = new PostgresOrderRepository(); // coupling!
}
```

### Composition root (container/factory)

All wiring happens in **one place**, at the outermost edge — often called the composition root, `main`, or container:

```ts
// main.ts — the only file allowed to know about everything
function buildApp(config: Config) {
  const pool     = createPool(config.db);
  const orders   = new PostgresOrderRepository(pool);
  const payments = new StripePaymentGateway(config.stripe);
  const notifier = new SmtpNotifier(config.smtp);

  const placeOrder = new PlaceOrder(orders, payments, notifier);
  return new HttpServer({ orderController: new OrderController(placeOrder) });
}
```

### Runtime vs. compile-time resolution

- **Compile-time (manual wiring or DI frameworks resolving at build/startup):** explicit, traceable, type-safe. Prefer this default.
- **Runtime (config-driven selection, plugin loading, reflection containers):** choose the adapter from configuration at startup (`STORAGE=s3` vs `STORAGE=local`). Powerful for plugins and multi-environment deploys, but keep the dynamic part small and fail fast at startup if a binding is missing — never mid-request.

---

## 7. Structuring Code

### Folder organization

```
src/
├── domain/                  # inside the hexagon
│   ├── model/               #   entities, value objects, domain errors
│   ├── usecases/            #   application services / workflows
│   └── ports/               #   ALL interfaces (owned by the core)
│       ├── driven/          #     repositories, gateways, notifiers
│       └── driving/         #     use-case interfaces (optional)
├── adapters/                # outside the hexagon
│   ├── driving/
│   │   ├── http/            #   controllers, routes, request mappers
│   │   ├── cli/
│   │   └── mcp/
│   └── driven/
│       ├── persistence/     #   postgres/, in-memory/
│       ├── payment/         #   stripe/
│       └── notification/    #   smtp/
└── main.ts                  # composition root
```

For larger systems, organize **by feature first, then by layer** (`orders/domain`, `orders/adapters`, `billing/domain`, ...) to keep modules cohesive.

### Module boundaries and what goes where

| Concern | Belongs in | Common mistake |
|---|---|---|
| Validation of business invariants | Domain | Put in controller "since we're validating anyway" |
| Request-shape validation (types, required fields) | Driving adapter | Domain parsing JSON |
| SQL / ORM mappings | Driven adapter | ORM entities used as domain models |
| Retry/timeout policy for a vendor API | Driven adapter | Use case sprinkled with retry loops |
| Transaction orchestration | Use case (via a port like `UnitOfWork`) | Controller opening transactions |
| DTO ↔ domain mapping | Adapter | Domain importing DTO types |

### Preventing leaky abstractions

- Ports return **domain types**, never library types (`Order`, not `PrismaOrder`; `PaymentResult`, not `Stripe.Charge`).
- Port errors are **domain errors** (`PaymentDeclined`), never vendor exceptions.
- Enforce with tooling: lint rules / import boundaries (`dependency-cruiser`, ESLint `no-restricted-imports`, ArchUnit for JVM) that fail the build if `domain/` imports from `adapters/` or from vendor packages.

&gt; **Warning:** The most common leak is the ORM entity doubling as the domain model. It feels DRY; it welds your business rules to your persistence framework. Keep them separate and map in the adapter.

---

## 8. Common Pitfalls

1. **Adapter logic bleeding into domain.** Pagination tokens, HTTP status codes, or SQL-ish query objects appearing in port signatures. *Symptom:* changing the database forces a port change. *Fix:* re-express the need in domain terms.

2. **Ports shaped by one implementation.** `OrderRepository.executeRawQuery(sql)` is Postgres wearing an interface as a costume. If only one conceivable implementation could satisfy the port, it's not an abstraction.

3. **Over-abstracting.** A port for something that will *never* be swapped and has no testing benefit is ceremony. You don't need `IStringUtils`. Rule of thumb: create a port when the dependency does I/O, is nondeterministic, is a vendor, or you genuinely need to fake it in tests.

4. **Tight coupling despite the pattern.** All the folders exist, but the use case assumes save-then-immediately-read consistency, ordering guarantees, or specific error timing that only one adapter provides. The contract is implicit and untested. *Fix:* contract tests shared across implementations.

5. **Anemic pass-through layers.** Controllers calling services calling managers calling repositories, each adding nothing. Hexagonal is about *boundaries*, not layer count.

6. **Testing pitfalls.**
   - **Mocking the world:** deep mock setups that mirror implementation details make tests brittle. Prefer simple hand-written in-memory fakes.
   - **Only unit tests:** a perfectly tested domain wired to untested adapters still breaks in production. Keep adapter integration tests and a few end-to-end smoke tests.
   - **Testing through the wrong boundary:** unit-test business rules via use cases, not through HTTP.

---

## 9. Best Practices

- **Keep ports minimal.** Add methods when a use case needs them, not speculatively. A port with 20 methods is usually three ports.
- **Adapters own their details.** Connection pooling, retries, caching, serialization — invisible to the core.
- **Inject consistently.** One composition root; no `new`-ing adapters inside the domain, no service locators sprinkled through business code.
- **Test the domain with fakes.** A hand-rolled `InMemoryOrderRepository` (a `Map` behind the interface) beats a mocking framework for readability and reuse — and it can double as your local-dev adapter.
- **Document port contracts.** For each port: purpose, method semantics, error conditions, and guarantees (idempotency? ordering? consistency?). The interface signature alone is not the contract.
- **Configuration is a first-class concern.** Adapters receive validated, typed config at construction; the domain never reads environment variables. Validate all config at startup and fail fast.
- **Map errors at the boundary.** Every adapter translates infrastructure failures into the port's declared error types.

---

## 10. Extensibility &amp; Plugin Architecture

Ports &amp; Adapters is a natural plugin system: the port *is* the plugin API.

### Designing for custom adapters

- Publish the port interfaces (and their domain types) as a small, dependency-light package.
- Ship a **contract test kit** plugin authors run against their implementation.
- Keep the port surface stable and minimal — every method is a promise to every plugin author.

### Configuration-driven selection

```yaml
storage:
  adapter: s3            # or "local", "gcs", "custom"
  options: { bucket: my-bucket, region: us-east-1 }
```

```ts
// composition root
const registry: Record&lt;string, (opts: unknown) =&gt; StoragePort&gt; = {
  s3:    (o) =&gt; new S3Storage(parseS3Options(o)),
  local: (o) =&gt; new LocalFsStorage(parseLocalOptions(o)),
};
const storage = registry[config.storage.adapter]?.(config.storage.options)
  ?? fail(`Unknown storage adapter: ${config.storage.adapter}`);
```

### Loading custom implementations at runtime

For true third-party plugins: dynamic import of a module that default-exports a factory conforming to `(options) =&gt; StoragePort`. Validate the loaded object implements the port (method presence, a self-check call) at startup — fail fast, never on first use. Treat plugin code as untrusted to whatever degree your threat model requires.

### Versioning adapter interfaces

- Treat port changes as **semver events**: additive optional methods are minor; signature changes are major.
- Prefer *extending* via new, optional capability interfaces (`interface StreamingStorage extends Storage`) over breaking the base port; feature-detect in the core.
- When you must break: version the port (`StoragePortV2`), support both during a deprecation window, and provide a shim adapter (V1→V2) so old plugins keep working.

---

## 11. When to Use (and Not Use)

### Good fit

- Systems with **real business logic** that will live for years.
- Multiple delivery mechanisms — HTTP + CLI + MCP tools + queue consumers over one core.
- Known or likely **infrastructure change** (vendor migrations, multi-cloud, on-prem + SaaS deployments).
- Products needing a **plugin ecosystem**.
- Teams that value fast, deterministic test suites.

### Overkill

- Thin CRUD apps where the "business logic" is `save the form`. The pattern's layers would outweigh the logic they protect.
- Prototypes and throwaway scripts.
- Pure glue/ETL jobs that are *entirely* adapter — there's no core to protect.

### Scaling from simple to complex

You don't need the full folder ceremony on day one. A pragmatic ramp:

1. **Start:** one rule — business logic in plain functions/classes that take their dependencies as parameters. That's the seed of DI.
2. **Grow:** when a second implementation or a test fake appears, extract the interface (a port is born).
3. **Formalize:** when multiple people work in the code, adopt the folder structure and add import-boundary linting.
4. **Extend:** when outsiders need to add behavior, harden the ports into a versioned plugin API.

The pattern scales down gracefully: even "inject the database client instead of importing it" buys most of the testing benefit.

---

## 12. Real-World Example Structure

A hypothetical **invoicing service** (HTTP API + CLI, Postgres, Stripe, email):

```
invoicing/
├── domain/
│   ├── model/
│   │   ├── invoice.ts             # Invoice aggregate: line items, status rules
│   │   └── money.ts               # value object
│   ├── usecases/
│   │   ├── issue-invoice.ts       # validate → persist → charge → notify
│   │   └── void-invoice.ts
│   └── ports/
│       ├── invoice-repository.ts
│       ├── payment-gateway.ts
│       ├── invoice-notifier.ts
│       └── clock.ts
├── adapters/
│   ├── driving/
│   │   ├── http/invoice-controller.ts
│   │   └── cli/issue-invoice-command.ts
│   └── driven/
│       ├── persistence/
│       │   ├── postgres-invoice-repository.ts
│       │   └── in-memory-invoice-repository.ts   # tests + local dev
│       ├── payment/stripe-payment-gateway.ts
│       └── notification/smtp-invoice-notifier.ts
├── tests/
│   ├── domain/issue-invoice.test.ts        # fakes only, milliseconds
│   ├── contracts/invoice-repository.contract.ts  # run vs postgres AND in-memory
│   ├── adapters/postgres-invoice-repository.int.test.ts  # Testcontainers
│   └── e2e/issue-invoice.e2e.test.ts       # full wiring, happy path
└── main.ts
```

### How a change flows through the layers

**New rule: "invoices over $10,000 require a second approver."**

1. `domain/model/invoice.ts` — add the invariant to `Invoice.issue()`.
2. `domain/usecases/issue-invoice.ts` — handle the new `ApprovalRequired` outcome.
3. `adapters/driving/http` — map `ApprovalRequired` to a `409` response.
4. Ports, database adapter, Stripe adapter: **untouched.**

**New requirement: "migrate from Stripe to Adyen."**

1. Write `adyen-payment-gateway.ts` implementing `PaymentGateway`.
2. Run the payment-gateway contract tests against it.
3. Change one line in the composition root (or a config value).
4. Domain, use cases, controllers: **untouched.**

That asymmetry — business changes touch only the core, infrastructure changes touch only adapters — is the pattern working as intended.

### Testing at each layer

| Layer | Test style | Speed | What's real |
|---|---|---|---|
| Domain model | Pure unit tests | ms | Everything (it's pure) |
| Use cases | Unit tests with in-memory fakes | ms | Domain + fakes |
| Driven adapters | Integration (Testcontainers, sandboxes) | seconds | The technology |
| Port contracts | Shared suite vs. every implementation | mixed | Contract semantics |
| Driving adapters | Translation tests, use case faked | ms | Parsing/mapping |
| End-to-end | Few smoke tests, full wiring | slow | Everything |

---

## Quick Reference

- **Port** = interface, owned by the domain, in domain language.
- **Adapter** = implementation, owns its technology, translates at the boundary.
- **Driving** = calls the core (HTTP, CLI, MCP, tests). **Driven** = called by the core (DB, APIs, email).
- **Dependencies point inward** at compile time, always.
- **Wire once**, in a composition root; inject everywhere else.
- **Fake ports in domain tests; integration-test adapters; contract-test both.**
- **Don't abstract what you'll never swap and never need to fake.**