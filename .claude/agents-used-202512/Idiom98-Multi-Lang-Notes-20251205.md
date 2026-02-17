
# S01: Polyglot Development - Core Principles & TDD Workflow

## Philosophy: Idiomatic Code Across Technology Stacks

This documentation provides production-ready patterns for **TypeScript/JavaScript**, **Java/Spring Boot**, **Golang**, **DevOps**, and **Apache Kafka** following the structure:

**Use When → Context → Avoid/Anti-pattern**

With TDD-first approach: **STUB → RED → GREEN → REFACTOR**

---

## CRITICAL: NAMING CONVENTIONS (LLM-Optimized)

### TypeScript/JavaScript
| Context | Convention | Example |
|---------|------------|---------|
| Functions | camelCase, verb-first | `fetchUserById()` |
| Classes | PascalCase | `UserRepository` |
| Interfaces | PascalCase, no `I` prefix | `UserService` |
| Type aliases | PascalCase | `UserId` |
| Constants | SCREAMING_SNAKE | `MAX_RETRY_COUNT` |
| Files (components) | PascalCase | `UserProfile.tsx` |
| Files (utilities) | kebab-case | `date-utils.ts` |
| Test files | `*.test.ts` or `*.spec.ts` | `user.service.test.ts` |

### Java/Spring Boot
| Context | Convention | Example |
|---------|------------|---------|
| Classes | PascalCase | `UserService` |
| Interfaces | PascalCase, no `I` prefix | `UserRepository` |
| Methods | camelCase, verb-first | `findUserById()` |
| Constants | SCREAMING_SNAKE | `MAX_CONNECTIONS` |
| Packages | lowercase, reverse domain | `com.company.users` |
| Test classes | `*Test` or `*IT` | `UserServiceTest` |
| Spring beans | camelCase | `userService` |

### Golang
| Context | Convention | Example |
|---------|------------|---------|
| Exported | PascalCase | `UserService` |
| Unexported | camelCase | `parseConfig` |
| Packages | lowercase, single word | `users` |
| Interfaces | -er suffix for single method | `Reader`, `Closer` |
| Files | snake_case | `user_service.go` |
| Test files | `*_test.go` | `user_service_test.go` |
| Acronyms | All caps | `HTTPClient`, `ID` |

### DevOps/Infrastructure
| Context | Convention | Example |
|---------|------------|---------|
| Terraform resources | snake_case | `aws_instance.web_server` |
| K8s resources | kebab-case | `user-service-deployment` |
| Docker images | lowercase/kebab | `company/user-service:v1.2.3` |
| Environment vars | SCREAMING_SNAKE | `DATABASE_URL` |
| Helm values | camelCase | `replicaCount` |
| GitHub Actions | kebab-case | `build-and-deploy.yml` |

### Apache Kafka
| Context | Convention | Example |
|---------|------------|---------|
| Topics | kebab-case with domain | `orders.created.v1` |
| Consumer groups | kebab-case with purpose | `order-processor-group` |
| Schemas (Avro) | PascalCase | `OrderCreatedEvent` |
| Connectors | PascalCase | `PostgresSourceConnector` |

---

## TDD WORKFLOW: STUB → RED → GREEN → REFACTOR

### Phase 1: STUB (Define the Contract)

**TypeScript:**
```typescript
// Define the interface FIRST
interface UserService {
  createUser(data: CreateUserDTO): Promise<Result<User, UserError>>;
  findById(id: UserId): Promise<User | null>;
  updateUser(id: UserId, data: UpdateUserDTO): Promise<Result<User, UserError>>;
}

// Define types
type UserId = string & { readonly brand: unique symbol };
type CreateUserDTO = { email: string; name: string };
type UserError = 'EMAIL_EXISTS' | 'INVALID_DATA' | 'NOT_FOUND';
```

**Java:**
```java
// Define the interface FIRST
public interface UserService {
    Result<User, UserError> createUser(CreateUserDTO data);
    Optional<User> findById(UserId id);
    Result<User, UserError> updateUser(UserId id, UpdateUserDTO data);
}

// Define value objects
public record UserId(String value) {
    public UserId {
        Objects.requireNonNull(value);
        if (value.isBlank()) throw new IllegalArgumentException("UserId cannot be blank");
    }
}
```

**Go:**
```go
// Define the interface FIRST
type UserService interface {
    CreateUser(ctx context.Context, data CreateUserDTO) (*User, error)
    FindByID(ctx context.Context, id UserID) (*User, error)
    UpdateUser(ctx context.Context, id UserID, data UpdateUserDTO) (*User, error)
}

// Define types
type UserID string

type CreateUserDTO struct {
    Email string `json:"email" validate:"required,email"`
    Name  string `json:"name" validate:"required,min=1,max=100"`
}
```

### Phase 2: RED (Write Failing Test)

**TypeScript (Vitest):**
```typescript
describe('UserService', () => {
  it('should create user with valid data', async () => {
    const service = new UserServiceImpl(mockRepo, mockEventBus);
    const dto: CreateUserDTO = { email: 'test@example.com', name: 'Test' };
    
    const result = await service.createUser(dto);
    
    expect(result.isOk()).toBe(true);
    expect(result.unwrap().email).toBe('test@example.com');
  });
});
// Test FAILS - UserServiceImpl not implemented
```

**Java (JUnit 5):**
```java
@ExtendWith(MockitoExtension.class)
class UserServiceTest {
    @Mock UserRepository repository;
    @Mock EventPublisher eventPublisher;
    @InjectMocks UserServiceImpl service;

    @Test
    void shouldCreateUserWithValidData() {
        var dto = new CreateUserDTO("test@example.com", "Test");
        when(repository.existsByEmail(dto.email())).thenReturn(false);
        when(repository.save(any())).thenAnswer(inv -> inv.getArgument(0));

        var result = service.createUser(dto);

        assertThat(result.isSuccess()).isTrue();
        assertThat(result.getValue().email()).isEqualTo("test@example.com");
    }
}
// Test FAILS - UserServiceImpl not implemented
```

**Go:**
```go
func TestUserService_CreateUser(t *testing.T) {
    ctrl := gomock.NewController(t)
    defer ctrl.Finish()
    
    mockRepo := mocks.NewMockUserRepository(ctrl)
    mockEvents := mocks.NewMockEventPublisher(ctrl)
    service := NewUserService(mockRepo, mockEvents)
    
    dto := CreateUserDTO{Email: "test@example.com", Name: "Test"}
    mockRepo.EXPECT().ExistsByEmail(gomock.Any(), dto.Email).Return(false, nil)
    mockRepo.EXPECT().Save(gomock.Any(), gomock.Any()).Return(nil)
    
    user, err := service.CreateUser(context.Background(), dto)
    
    require.NoError(t, err)
    assert.Equal(t, "test@example.com", user.Email)
}
// Test FAILS - UserService not implemented
```

### Phase 3: GREEN (Minimal Implementation)

**TypeScript:**
```typescript
class UserServiceImpl implements UserService {
  constructor(
    private readonly repo: UserRepository,
    private readonly events: EventBus
  ) {}

  async createUser(data: CreateUserDTO): Promise<Result<User, UserError>> {
    if (await this.repo.existsByEmail(data.email)) {
      return Result.err('EMAIL_EXISTS');
    }
    
    const user = User.create(data);
    await this.repo.save(user);
    await this.events.publish(new UserCreatedEvent(user));
    
    return Result.ok(user);
  }
}
// Test PASSES
```

**Java:**
```java
@Service
@RequiredArgsConstructor
public class UserServiceImpl implements UserService {
    private final UserRepository repository;
    private final EventPublisher eventPublisher;

    @Override
    @Transactional
    public Result<User, UserError> createUser(CreateUserDTO data) {
        if (repository.existsByEmail(data.email())) {
            return Result.failure(UserError.EMAIL_EXISTS);
        }
        
        var user = User.create(data);
        repository.save(user);
        eventPublisher.publish(new UserCreatedEvent(user));
        
        return Result.success(user);
    }
}
// Test PASSES
```

**Go:**
```go
type userServiceImpl struct {
    repo   UserRepository
    events EventPublisher
}

func NewUserService(repo UserRepository, events EventPublisher) UserService {
    return &userServiceImpl{repo: repo, events: events}
}

func (s *userServiceImpl) CreateUser(ctx context.Context, dto CreateUserDTO) (*User, error) {
    exists, err := s.repo.ExistsByEmail(ctx, dto.Email)
    if err != nil {
        return nil, fmt.Errorf("checking email existence: %w", err)
    }
    if exists {
        return nil, ErrEmailExists
    }
    
    user := NewUser(dto)
    if err := s.repo.Save(ctx, user); err != nil {
        return nil, fmt.Errorf("saving user: %w", err)
    }
    
    s.events.Publish(ctx, UserCreatedEvent{User: user})
    return user, nil
}
// Test PASSES
```

### Phase 4: REFACTOR (Production Quality)

Add:
- Input validation
- Structured logging
- Metrics
- Error context
- Documentation

---

## THE TESTING TROPHY (Kent C. Dodds Philosophy)

```
         ___________
        /   E2E     \      ← Critical user paths only (Playwright/Cypress)
       /─────────────\
      /  Integration   \    ← HIGHEST ROI: Components working together
     /─────────────────\
    /       Unit        \   ← Complex algorithms, pure functions
   /─────────────────────\
  /        Static         \  ← TypeScript/Go compiler, ESLint, Checkstyle
 /─────────────────────────\
```

**"Write tests. Not too many. Mostly integration."**

---

## LAYERED ARCHITECTURE (All Technologies)

```
┌─────────────────────────────────────────────────────────────┐
│                    PRESENTATION LAYER                        │
│    Controllers, Handlers, GraphQL Resolvers, CLI Commands    │
└─────────────────────────────────────────────────────────────┘
                            │
                            ▼
┌─────────────────────────────────────────────────────────────┐
│                    APPLICATION LAYER                         │
│         Services, Use Cases, Commands, Queries, DTOs         │
└─────────────────────────────────────────────────────────────┘
                            │
                            ▼
┌─────────────────────────────────────────────────────────────┐
│                      DOMAIN LAYER                            │
│        Entities, Value Objects, Domain Events, Aggregates    │
└─────────────────────────────────────────────────────────────┘
                            │
                            ▼
┌─────────────────────────────────────────────────────────────┐
│                  INFRASTRUCTURE LAYER                        │
│      Repositories, External APIs, Message Queues, Caches     │
└─────────────────────────────────────────────────────────────┘
```

### Dependency Rule
- Inner layers NEVER depend on outer layers
- Dependencies point INWARD
- Infrastructure implements Domain interfaces

---

## EXECUTABLE SPECIFICATIONS PATTERN

### Format: WHEN...THEN...SHALL

```gherkin
Feature: User Registration

Scenario: Successful registration with valid email
  GIVEN a user with email "new@example.com" does not exist
  WHEN I submit registration with email "new@example.com" and password "SecurePass123!"
  THEN the system SHALL create a new user account
  AND the system SHALL send a verification email
  AND the system SHALL return status 201 with user ID

Scenario: Registration fails with existing email
  GIVEN a user with email "existing@example.com" already exists
  WHEN I submit registration with email "existing@example.com"
  THEN the system SHALL return status 409 CONFLICT
  AND the system SHALL NOT create a duplicate account
```

---

## QUALITY CHECKLISTS

### ✅ Code Quality (All Languages)
- [ ] All public APIs have documentation
- [ ] No magic numbers (use named constants)
- [ ] Functions do ONE thing
- [ ] Error messages are actionable
- [ ] No commented-out code
- [ ] Consistent formatting (automated)

### ✅ Testing Quality
- [ ] Tests follow AAA pattern (Arrange, Act, Assert)
- [ ] Test names describe behavior, not implementation
- [ ] No test interdependence
- [ ] Mocks only at boundaries
- [ ] Coverage > 80% for business logic
- [ ] No flaky tests in CI

### ✅ Security Checklist
- [ ] Input validation at boundaries
- [ ] No secrets in code
- [ ] SQL/NoSQL injection prevention
- [ ] Authentication on all sensitive endpoints
- [ ] Rate limiting configured
- [ ] CORS properly configured
- [ ] Dependencies scanned for vulnerabilities

### ✅ Observability Checklist
- [ ] Structured logging (JSON)
- [ ] Request tracing (correlation IDs)
- [ ] Metrics for SLIs (latency, errors, throughput)
- [ ] Health check endpoints
- [ ] Graceful shutdown
- [ ] Alert thresholds defined

---

## ERROR HANDLING PHILOSOPHY

### The Result/Either Pattern (Recommended)

**TypeScript:**
```typescript
type Result<T, E> = { ok: true; value: T } | { ok: false; error: E };

// Usage
function divide(a: number, b: number): Result<number, 'DIVISION_BY_ZERO'> {
  if (b === 0) return { ok: false, error: 'DIVISION_BY_ZERO' };
  return { ok: true, value: a / b };
}
```

**Java:**
```java
public sealed interface Result<T, E> {
    record Success<T, E>(T value) implements Result<T, E> {}
    record Failure<T, E>(E error) implements Result<T, E> {}
    
    static <T, E> Result<T, E> success(T value) { return new Success<>(value); }
    static <T, E> Result<T, E> failure(E error) { return new Failure<>(error); }
}
```

**Go:**
```go
// Go uses multiple return values - this IS the Result pattern
func divide(a, b float64) (float64, error) {
    if b == 0 {
        return 0, errors.New("division by zero")
    }
    return a / b, nil
}
```

### Error Categories

| Category | HTTP Status | Retry? | Log Level |
|----------|-------------|--------|-----------|
| Validation | 400 | No | WARN |
| Authentication | 401 | No | WARN |
| Authorization | 403 | No | WARN |
| Not Found | 404 | No | DEBUG |
| Conflict | 409 | No | INFO |
| Rate Limited | 429 | Yes (backoff) | WARN |
| Server Error | 500 | Yes (limited) | ERROR |
| Unavailable | 503 | Yes (backoff) | ERROR |

---

## DIAGRAMS: MERMAID ONLY

All architecture diagrams use Mermaid for GitHub compatibility:

```mermaid
flowchart TD
    A[Client] -->|HTTP/gRPC| B[API Gateway]
    B -->|Route| C[User Service]
    B -->|Route| D[Order Service]
    C -->|Query/Command| E[(PostgreSQL)]
    D -->|Query/Command| F[(PostgreSQL)]
    C -->|Publish| G[Kafka]
    D -->|Consume| G
    D -->|Cache| H[(Redis)]
```

---

## TECHNOLOGY-SPECIFIC PRINCIPLES

### TypeScript: Type Safety First
- Enable `strict: true` always
- Prefer `unknown` over `any`
- Use branded types for domain IDs
- Leverage discriminated unions for state machines

### Java: Immutability by Default
- Use `record` for DTOs and value objects
- Prefer `final` fields
- Use `Optional` for nullable returns (never for parameters)
- Leverage sealed classes for domain modeling

### Go: Simplicity and Explicitness
- Accept interfaces, return structs
- Handle errors immediately
- Use `context.Context` for cancellation/timeouts
- Prefer composition over inheritance

### DevOps: Cattle Not Pets
- Infrastructure as Code (no manual changes)
- Immutable deployments
- Rollback capability always
- Observe everything

### Kafka: Event-Driven Thinking
- Events are facts (immutable)
- Design for replay
- Schema evolution from day 1
- Idempotent consumers always


# S06: TDD-First Architecture Principles
## Polyglot Edition: TypeScript • Java • Go • DevOps • Kafka

---

## The Essence: Executable Specifications Drive Everything

**Core Truth**: Traditional user stories fail LLMs because they're designed for human conversation. LLMs need executable blueprints, not ambiguous narratives.

**The Solution**: Transform all specifications into formal, testable contracts with preconditions, postconditions, and error conditions. Every claim must be validated by automated tests.

**Why This Matters**: Eliminates the #1 cause of LLM hallucination - ambiguous requirements that lead to incorrect implementations.

---

## The STUB → RED → GREEN → REFACTOR Cycle

### Phase 1: STUB (Define the Contract)

Write the interface/type BEFORE any implementation:

```typescript
// TypeScript: Define types and interfaces first
interface OrderService {
  createOrder(request: CreateOrderRequest): Promise<Result<Order, OrderError>>;
  cancelOrder(orderId: OrderId, reason: string): Promise<Result<void, OrderError>>;
  getOrderStatus(orderId: OrderId): Promise<OrderStatus | null>;
}

type OrderError = 
  | { code: 'INSUFFICIENT_STOCK'; productId: string; available: number }
  | { code: 'INVALID_PAYMENT'; message: string }
  | { code: 'ORDER_NOT_FOUND' };
```

```java
// Java: Define interface with contracts in Javadoc
/**
 * Service for managing orders.
 *
 * <h2>Invariants</h2>
 * <ul>
 *   <li>Orders are immutable once placed - modifications create new versions</li>
 *   <li>Order IDs are globally unique UUIDs</li>
 *   <li>All monetary values use BigDecimal with 2 decimal places</li>
 * </ul>
 */
public interface OrderService {
    /**
     * Creates a new order.
     *
     * <h3>Preconditions</h3>
     * <ul>
     *   <li>Customer exists and is active</li>
     *   <li>All products in request are available</li>
     *   <li>Quantities are positive integers</li>
     * </ul>
     *
     * <h3>Postconditions</h3>
     * <ul>
     *   <li>Order persisted with PENDING status</li>
     *   <li>Inventory reserved for all items</li>
     *   <li>OrderCreatedEvent published</li>
     * </ul>
     *
     * @throws InsufficientStockException if any product lacks inventory
     * @throws CustomerNotFoundException if customer doesn't exist
     */
    Result<Order, OrderError> createOrder(CreateOrderRequest request);
}
```

```go
// Go: Define interface with comments
// OrderService manages the order lifecycle.
//
// Thread Safety: All methods are safe for concurrent use.
// Context: All methods respect context cancellation and deadlines.
type OrderService interface {
    // CreateOrder places a new order.
    //
    // Preconditions:
    //   - ctx is not cancelled
    //   - request passes validation
    //
    // Postconditions:
    //   - Order persisted with StatusPending
    //   - Inventory reserved
    //   - OrderCreatedEvent published
    //
    // Errors:
    //   - ErrInsufficientStock: product lacks inventory
    //   - ErrCustomerNotFound: customer doesn't exist
    //   - ErrValidation: request validation failed
    CreateOrder(ctx context.Context, request CreateOrderRequest) (*Order, error)
    
    // CancelOrder cancels an existing order.
    CancelOrder(ctx context.Context, orderID OrderID, reason string) error
    
    // GetOrderStatus returns current order status or nil if not found.
    GetOrderStatus(ctx context.Context, orderID OrderID) (*OrderStatus, error)
}
```

### Phase 2: RED (Write Failing Test)

Write the test BEFORE the implementation. The test MUST fail initially.

```typescript
// TypeScript: Test with Vitest
describe('OrderService', () => {
  let service: OrderService;
  let mockRepo: MockedObject<OrderRepository>;
  let mockInventory: MockedObject<InventoryService>;
  let mockEvents: MockedObject<EventBus>;

  beforeEach(() => {
    mockRepo = vi.mocked(createMockOrderRepository());
    mockInventory = vi.mocked(createMockInventoryService());
    mockEvents = vi.mocked(createMockEventBus());
    service = new OrderServiceImpl(mockRepo, mockInventory, mockEvents);
  });

  describe('createOrder', () => {
    it('should create order when stock is available', async () => {
      // Arrange
      const request: CreateOrderRequest = {
        customerId: CustomerId('cust-123'),
        items: [
          { productId: ProductId('prod-1'), quantity: 2 },
          { productId: ProductId('prod-2'), quantity: 1 },
        ],
      };
      
      mockInventory.checkAvailability.mockResolvedValue({ available: true });
      mockInventory.reserve.mockResolvedValue({ reserved: true });
      mockRepo.save.mockImplementation(async (order) => order);

      // Act
      const result = await service.createOrder(request);

      // Assert
      expect(result.ok).toBe(true);
      if (result.ok) {
        expect(result.value.status).toBe('PENDING');
        expect(result.value.items).toHaveLength(2);
      }
      
      expect(mockInventory.reserve).toHaveBeenCalledWith(
        expect.arrayContaining([
          { productId: 'prod-1', quantity: 2 },
          { productId: 'prod-2', quantity: 1 },
        ])
      );
      
      expect(mockEvents.publish).toHaveBeenCalledWith(
        expect.objectContaining({
          type: 'OrderCreated',
          orderId: expect.any(String),
        })
      );
    });

    it('should return INSUFFICIENT_STOCK error when product unavailable', async () => {
      // Arrange
      const request: CreateOrderRequest = {
        customerId: CustomerId('cust-123'),
        items: [{ productId: ProductId('prod-1'), quantity: 100 }],
      };
      
      mockInventory.checkAvailability.mockResolvedValue({ 
        available: false,
        productId: 'prod-1',
        currentStock: 5,
      });

      // Act
      const result = await service.createOrder(request);

      // Assert
      expect(result.ok).toBe(false);
      if (!result.ok) {
        expect(result.error.code).toBe('INSUFFICIENT_STOCK');
        expect(result.error.productId).toBe('prod-1');
        expect(result.error.available).toBe(5);
      }
      
      // Verify no side effects
      expect(mockRepo.save).not.toHaveBeenCalled();
      expect(mockEvents.publish).not.toHaveBeenCalled();
    });
  });
});
// Test FAILS - OrderServiceImpl not implemented
```

```java
// Java: Test with JUnit 5 + Mockito
@ExtendWith(MockitoExtension.class)
class OrderServiceTest {

    @Mock OrderRepository repository;
    @Mock InventoryService inventoryService;
    @Mock EventPublisher eventPublisher;
    
    @InjectMocks OrderServiceImpl service;

    @Nested
    @DisplayName("createOrder")
    class CreateOrder {
        
        @Test
        @DisplayName("should create order when stock is available")
        void shouldCreateOrderWhenStockAvailable() {
            // Arrange
            var request = new CreateOrderRequest(
                new CustomerId("cust-123"),
                List.of(
                    new OrderItem(new ProductId("prod-1"), 2),
                    new OrderItem(new ProductId("prod-2"), 1)
                )
            );
            
            when(inventoryService.checkAvailability(any()))
                .thenReturn(new AvailabilityResult(true, Map.of()));
            when(inventoryService.reserve(any()))
                .thenReturn(new ReservationResult(true, "res-123"));
            when(repository.save(any()))
                .thenAnswer(inv -> inv.getArgument(0));
            
            // Act
            var result = service.createOrder(request);
            
            // Assert
            assertThat(result.isSuccess()).isTrue();
            assertThat(result.getValue().status()).isEqualTo(OrderStatus.PENDING);
            assertThat(result.getValue().items()).hasSize(2);
            
            verify(inventoryService).reserve(argThat(items -> 
                items.size() == 2 &&
                items.stream().anyMatch(i -> i.productId().equals("prod-1"))
            ));
            
            verify(eventPublisher).publish(argThat(event ->
                event instanceof OrderCreatedEvent &&
                ((OrderCreatedEvent) event).orderId() != null
            ));
        }
        
        @Test
        @DisplayName("should return INSUFFICIENT_STOCK error when product unavailable")
        void shouldReturnErrorWhenStockInsufficient() {
            // Arrange
            var request = new CreateOrderRequest(
                new CustomerId("cust-123"),
                List.of(new OrderItem(new ProductId("prod-1"), 100))
            );
            
            when(inventoryService.checkAvailability(any()))
                .thenReturn(new AvailabilityResult(false, Map.of("prod-1", 5)));
            
            // Act
            var result = service.createOrder(request);
            
            // Assert
            assertThat(result.isFailure()).isTrue();
            assertThat(result.getError())
                .isInstanceOf(InsufficientStockError.class)
                .extracting("productId", "available")
                .containsExactly("prod-1", 5);
            
            // Verify no side effects
            verify(repository, never()).save(any());
            verify(eventPublisher, never()).publish(any());
        }
    }
}
// Test FAILS - OrderServiceImpl not implemented
```

```go
// Go: Test with testify
func TestOrderService_CreateOrder(t *testing.T) {
    tests := []struct {
        name          string
        request       CreateOrderRequest
        setupMocks    func(*mocks.MockOrderRepository, *mocks.MockInventoryService, *mocks.MockEventPublisher)
        wantErr       error
        wantStatus    OrderStatus
        wantItemCount int
    }{
        {
            name: "should create order when stock available",
            request: CreateOrderRequest{
                CustomerID: CustomerID("cust-123"),
                Items: []OrderItemRequest{
                    {ProductID: ProductID("prod-1"), Quantity: 2},
                    {ProductID: ProductID("prod-2"), Quantity: 1},
                },
            },
            setupMocks: func(repo *mocks.MockOrderRepository, inv *mocks.MockInventoryService, events *mocks.MockEventPublisher) {
                inv.EXPECT().
                    CheckAvailability(gomock.Any(), gomock.Any()).
                    Return(&AvailabilityResult{Available: true}, nil)
                inv.EXPECT().
                    Reserve(gomock.Any(), gomock.Any()).
                    Return(&ReservationResult{ReservationID: "res-123"}, nil)
                repo.EXPECT().
                    Save(gomock.Any(), gomock.Any()).
                    DoAndReturn(func(ctx context.Context, order *Order) error {
                        return nil
                    })
                events.EXPECT().
                    Publish(gomock.Any(), gomock.Any()).
                    Return(nil)
            },
            wantErr:       nil,
            wantStatus:    StatusPending,
            wantItemCount: 2,
        },
        {
            name: "should return error when stock insufficient",
            request: CreateOrderRequest{
                CustomerID: CustomerID("cust-123"),
                Items: []OrderItemRequest{
                    {ProductID: ProductID("prod-1"), Quantity: 100},
                },
            },
            setupMocks: func(repo *mocks.MockOrderRepository, inv *mocks.MockInventoryService, events *mocks.MockEventPublisher) {
                inv.EXPECT().
                    CheckAvailability(gomock.Any(), gomock.Any()).
                    Return(&AvailabilityResult{
                        Available:    false,
                        Unavailable:  map[ProductID]int{ProductID("prod-1"): 5},
                    }, nil)
                // Verify no side effects
                repo.EXPECT().Save(gomock.Any(), gomock.Any()).Times(0)
                events.EXPECT().Publish(gomock.Any(), gomock.Any()).Times(0)
            },
            wantErr: ErrInsufficientStock,
        },
    }

    for _, tt := range tests {
        t.Run(tt.name, func(t *testing.T) {
            ctrl := gomock.NewController(t)
            defer ctrl.Finish()

            mockRepo := mocks.NewMockOrderRepository(ctrl)
            mockInv := mocks.NewMockInventoryService(ctrl)
            mockEvents := mocks.NewMockEventPublisher(ctrl)

            tt.setupMocks(mockRepo, mockInv, mockEvents)

            service := NewOrderService(mockRepo, mockInv, mockEvents)
            order, err := service.CreateOrder(context.Background(), tt.request)

            if tt.wantErr != nil {
                require.Error(t, err)
                assert.ErrorIs(t, err, tt.wantErr)
                return
            }

            require.NoError(t, err)
            assert.Equal(t, tt.wantStatus, order.Status)
            assert.Len(t, order.Items, tt.wantItemCount)
        })
    }
}
// Test FAILS - OrderService implementation incomplete
```

### Phase 3: GREEN (Minimal Implementation)

Write the MINIMUM code to make the test pass:

```typescript
// TypeScript: Minimal implementation
class OrderServiceImpl implements OrderService {
  constructor(
    private readonly repo: OrderRepository,
    private readonly inventory: InventoryService,
    private readonly events: EventBus,
  ) {}

  async createOrder(request: CreateOrderRequest): Promise<Result<Order, OrderError>> {
    // Check availability
    const availability = await this.inventory.checkAvailability(request.items);
    if (!availability.available) {
      return {
        ok: false,
        error: {
          code: 'INSUFFICIENT_STOCK',
          productId: availability.productId!,
          available: availability.currentStock!,
        },
      };
    }

    // Reserve inventory
    await this.inventory.reserve(request.items);

    // Create and persist order
    const order: Order = {
      id: OrderId(crypto.randomUUID()),
      customerId: request.customerId,
      items: request.items,
      status: 'PENDING',
      createdAt: new Date(),
    };
    
    await this.repo.save(order);

    // Publish event
    await this.events.publish({
      type: 'OrderCreated',
      orderId: order.id,
      timestamp: new Date(),
    });

    return { ok: true, value: order };
  }
}
// Test PASSES
```

### Phase 4: REFACTOR (Production Quality)

Now improve the code while keeping tests green:

```typescript
// TypeScript: Refactored with proper error handling, logging, validation
class OrderServiceImpl implements OrderService {
  constructor(
    private readonly repo: OrderRepository,
    private readonly inventory: InventoryService,
    private readonly events: EventBus,
    private readonly logger: Logger,
  ) {}

  async createOrder(request: CreateOrderRequest): Promise<Result<Order, OrderError>> {
    const correlationId = generateCorrelationId();
    this.logger.info('Creating order', { correlationId, customerId: request.customerId });

    // Validate request
    const validation = this.validateRequest(request);
    if (!validation.ok) {
      this.logger.warn('Order validation failed', { correlationId, error: validation.error });
      return validation;
    }

    // Check availability
    const availability = await this.inventory.checkAvailability(request.items);
    if (!availability.available) {
      this.logger.info('Insufficient stock for order', { 
        correlationId,
        productId: availability.productId,
        available: availability.currentStock,
      });
      return {
        ok: false,
        error: {
          code: 'INSUFFICIENT_STOCK',
          productId: availability.productId!,
          available: availability.currentStock!,
        },
      };
    }

    // Reserve inventory with compensation on failure
    const reservation = await this.inventory.reserve(request.items);
    
    try {
      // Create order
      const order = Order.create({
        customerId: request.customerId,
        items: request.items.map(item => OrderItem.create(item)),
      });

      // Persist
      await this.repo.save(order);

      // Publish event (async, non-blocking)
      this.events.publish(OrderCreatedEvent.from(order)).catch(err => {
        this.logger.error('Failed to publish OrderCreated event', { 
          correlationId, 
          orderId: order.id,
          error: err,
        });
      });

      this.logger.info('Order created successfully', { correlationId, orderId: order.id });
      return { ok: true, value: order };

    } catch (error) {
      // Compensate: release reservation
      this.logger.error('Order creation failed, releasing reservation', { 
        correlationId,
        reservationId: reservation.id,
        error,
      });
      await this.inventory.releaseReservation(reservation.id);
      throw error;
    }
  }

  private validateRequest(request: CreateOrderRequest): Result<void, OrderError> {
    if (request.items.length === 0) {
      return { ok: false, error: { code: 'INVALID_REQUEST', message: 'Order must have at least one item' } };
    }
    
    for (const item of request.items) {
      if (item.quantity <= 0) {
        return { ok: false, error: { code: 'INVALID_REQUEST', message: 'Quantity must be positive' } };
      }
    }
    
    return { ok: true, value: undefined };
  }
}
// All tests still PASS
```

---

## Layered Architecture

### Dependency Direction

```
┌─────────────────────────────────────────────────────────────┐
│                    PRESENTATION LAYER                        │
│    HTTP Handlers / GraphQL Resolvers / gRPC Services        │
│                                                              │
│    Dependencies: Application Layer                           │
└─────────────────────────────────────────────────────────────┘
                            │
                            ▼
┌─────────────────────────────────────────────────────────────┐
│                    APPLICATION LAYER                         │
│            Services / Use Cases / Commands / Queries         │
│                                                              │
│    Dependencies: Domain Layer, Infrastructure Interfaces     │
└─────────────────────────────────────────────────────────────┘
                            │
                            ▼
┌─────────────────────────────────────────────────────────────┐
│                      DOMAIN LAYER                            │
│       Entities / Value Objects / Domain Events / Rules       │
│                                                              │
│    Dependencies: NONE (pure domain logic)                    │
└─────────────────────────────────────────────────────────────┘
                            △
                            │
┌─────────────────────────────────────────────────────────────┐
│                  INFRASTRUCTURE LAYER                        │
│    Repositories / Message Queues / External APIs / Cache     │
│                                                              │
│    Implements: Domain & Application interfaces               │
└─────────────────────────────────────────────────────────────┘
```

### Key Rules

1. **Domain has NO external dependencies** - pure business logic only
2. **Application orchestrates** - coordinates domain objects and infrastructure
3. **Infrastructure implements interfaces** - defined by domain/application
4. **Presentation transforms** - converts between external and internal representations

---

## Executable Specification Format

### WHEN...THEN...SHALL Format

```gherkin
Feature: Order Creation

Background:
  Given a customer "C-123" exists and is active
  And product "P-001" has 10 units in stock
  And product "P-002" has 5 units in stock

Scenario: Successfully create order with available inventory
  WHEN customer "C-123" submits an order with:
    | Product | Quantity |
    | P-001   | 2        |
    | P-002   | 1        |
  THEN the system SHALL create order with status "PENDING"
  AND the system SHALL reserve 2 units of "P-001"
  AND the system SHALL reserve 1 unit of "P-002"
  AND the system SHALL publish "OrderCreated" event

Scenario: Reject order when inventory insufficient
  GIVEN product "P-001" has only 1 unit in stock
  WHEN customer "C-123" submits an order with:
    | Product | Quantity |
    | P-001   | 5        |
  THEN the system SHALL reject order with error "INSUFFICIENT_STOCK"
  AND the error SHALL include product "P-001" with available quantity 1
  AND the system SHALL NOT reserve any inventory
  AND the system SHALL NOT publish any events
```

### Contract Documentation Pattern

```typescript
/**
 * Creates a new order for a customer.
 *
 * @preconditions
 * - Customer exists and is in ACTIVE status
 * - All products in the order exist and are purchasable
 * - Quantities are positive integers not exceeding MAX_QUANTITY (1000)
 * - Total order value does not exceed customer's credit limit
 *
 * @postconditions
 * - Order persisted with PENDING status
 * - Inventory reserved for all items
 * - OrderCreatedEvent published to message bus
 * - Audit log entry created
 *
 * @invariants
 * - Order total equals sum of (item.price × item.quantity) for all items
 * - Order ID is globally unique UUID v4
 * - createdAt timestamp is server time at creation
 *
 * @errors
 * - CUSTOMER_NOT_FOUND: Customer ID does not exist
 * - CUSTOMER_INACTIVE: Customer account is suspended or closed
 * - INSUFFICIENT_STOCK: One or more products lack inventory
 * - CREDIT_LIMIT_EXCEEDED: Order total exceeds customer credit limit
 * - VALIDATION_ERROR: Request data fails validation
 *
 * @performance
 * - P99 latency: <100ms for orders with ≤10 items
 * - P99 latency: <500ms for orders with ≤100 items
 *
 * @idempotency
 * - Requests with same idempotencyKey within 24h return cached result
 */
async createOrder(request: CreateOrderRequest): Promise<Result<Order, OrderError>>;
```

---

## Quality Assurance Checklists

### ✅ Test Quality Checklist

- [ ] Tests follow AAA pattern (Arrange, Act, Assert)
- [ ] Test names describe behavior: `should_[expected]_when_[condition]`
- [ ] One assertion per test (or cohesive assertion group)
- [ ] Tests are independent - no shared mutable state
- [ ] Tests run fast (<100ms for unit, <5s for integration)
- [ ] Tests are deterministic - no flaky tests
- [ ] Edge cases covered (null, empty, boundary values)
- [ ] Error paths tested, not just happy path
- [ ] Mocks verify interactions (called with correct arguments)
- [ ] No implementation details tested (internals, private methods)

### ✅ Code Quality Checklist

- [ ] Functions/methods do ONE thing
- [ ] No magic numbers - use named constants
- [ ] All public APIs documented with contracts
- [ ] Error messages are actionable
- [ ] Logging at appropriate levels with context
- [ ] No commented-out code
- [ ] Dependencies injected, not hard-coded
- [ ] Immutable by default

### ✅ Architecture Checklist

- [ ] Domain layer has zero external dependencies
- [ ] Infrastructure implements interfaces from domain/application
- [ ] No circular dependencies between layers
- [ ] Bounded contexts clearly separated
- [ ] Event-driven communication between contexts
- [ ] Database transactions don't span multiple aggregates

### ✅ Performance Checklist

- [ ] Performance requirements documented (P50, P99, throughput)
- [ ] Performance tests exist for critical paths
- [ ] N+1 queries eliminated
- [ ] Connection pooling configured
- [ ] Caching strategy defined for read-heavy operations
- [ ] Pagination implemented for list endpoints
- [ ] Async processing for non-critical operations

### ✅ Security Checklist

- [ ] Input validation at all boundaries
- [ ] SQL/NoSQL injection prevented
- [ ] Authentication on all sensitive endpoints
- [ ] Authorization checks before business operations
- [ ] Secrets not in code or logs
- [ ] Rate limiting configured
- [ ] CORS properly configured
- [ ] Dependencies scanned for vulnerabilities

### ✅ Observability Checklist

- [ ] Structured logging (JSON format)
- [ ] Correlation IDs propagated through requests
- [ ] Metrics exported (RED: Rate, Errors, Duration)
- [ ] Health check endpoints (/health, /ready)
- [ ] Distributed tracing for cross-service calls
- [ ] Alerts configured for SLO breaches
- [ ] Dashboards for key business metrics

---

## Anti-Patterns to Avoid

### ❌ The Fatal 10 Anti-Patterns

**1. God Object / God Service**
```typescript
// ❌ Bad: Does everything
class UserService {
  createUser() {}
  updateUser() {}
  deleteUser() {}
  sendEmail() {}
  chargePayment() {}
  generateReport() {}
  // 50 more methods...
}

// ✅ Good: Single responsibility
class UserService { createUser() {} updateUser() {} }
class EmailService { sendWelcome() {} sendPasswordReset() {} }
class PaymentService { charge() {} refund() {} }
```

**2. Anemic Domain Model**
```typescript
// ❌ Bad: Data bag with no behavior
class Order {
  id: string;
  items: OrderItem[];
  status: string;
  total: number;
}

// Service does all the work
class OrderService {
  calculateTotal(order: Order) { /*...*/ }
  addItem(order: Order, item: Item) { /*...*/ }
  canBeCancelled(order: Order) { /*...*/ }
}

// ✅ Good: Rich domain model
class Order {
  private items: OrderItem[];
  private _status: OrderStatus;
  
  get total(): Money {
    return this.items.reduce((sum, item) => sum.add(item.total), Money.zero());
  }
  
  addItem(product: Product, quantity: number): void {
    this.ensureModifiable();
    this.items.push(OrderItem.create(product, quantity));
  }
  
  cancel(): void {
    if (!this.canBeCancelled()) {
      throw new OrderCannotBeCancelledException(this.id, this._status);
    }
    this._status = OrderStatus.CANCELLED;
  }
  
  private canBeCancelled(): boolean {
    return this._status === OrderStatus.PENDING;
  }
}
```

**3. Testing Implementation Details**
```typescript
// ❌ Bad: Tests private method
it('should call validateEmail internally', () => {
  const spy = jest.spyOn(service, 'validateEmail' as any);
  service.createUser(data);
  expect(spy).toHaveBeenCalled();
});

// ✅ Good: Tests behavior
it('should reject invalid email format', async () => {
  const result = await service.createUser({ ...data, email: 'invalid' });
  expect(result.ok).toBe(false);
  expect(result.error.code).toBe('INVALID_EMAIL');
});
```

**4. Missing Error Handling**
```go
// ❌ Bad: Ignoring errors
user, _ := repo.FindByID(ctx, id)
data, _ := json.Marshal(user)

// ✅ Good: Handle all errors
user, err := repo.FindByID(ctx, id)
if err != nil {
    return nil, fmt.Errorf("finding user %s: %w", id, err)
}
```

**5. Temporal Coupling**
```typescript
// ❌ Bad: Must call methods in specific order
const processor = new OrderProcessor();
processor.loadOrder(orderId);      // Must be first
processor.validateInventory();      // Must be second
processor.calculateTotal();         // Must be third
processor.process();                // Finally

// ✅ Good: Self-contained operations
const result = await orderService.process(orderId);
```

**6. Leaky Abstractions**
```java
// ❌ Bad: Repository exposes JPA details
public interface UserRepository {
    @Query("SELECT u FROM User u WHERE u.email = :email")
    Optional<User> findByEmail(@Param("email") String email);
}

// ✅ Good: Clean domain interface
public interface UserRepository {
    Optional<User> findByEmail(Email email);
}
```

**7. Primitive Obsession**
```typescript
// ❌ Bad: Primitives everywhere
function createUser(email: string, name: string, age: number): User

// Confusing call site:
createUser("John", "john@example.com", 25); // Wrong order!

// ✅ Good: Value objects
function createUser(request: CreateUserRequest): User

type Email = string & { readonly brand: unique symbol };
type Name = string & { readonly brand: unique symbol };
```

**8. Service Locator / Hidden Dependencies**
```java
// ❌ Bad: Hidden dependency
public class UserService {
    public void createUser(CreateUserRequest request) {
        var repo = ServiceLocator.get(UserRepository.class); // Hidden!
        var events = ServiceLocator.get(EventPublisher.class);
    }
}

// ✅ Good: Explicit dependencies
public class UserService {
    private final UserRepository repository;
    private final EventPublisher events;
    
    public UserService(UserRepository repository, EventPublisher events) {
        this.repository = repository;
        this.events = events;
    }
}
```

**9. Boolean Parameters**
```typescript
// ❌ Bad: What does `true` mean?
sendEmail(user, true, false, true);

// ✅ Good: Named options
sendEmail(user, {
  includeAttachments: true,
  sendCopy: false,
  trackOpens: true,
});
```

**10. Shotgun Surgery**
```
// ❌ Bad: Adding a field requires changes in 10+ files
User.java
UserDTO.java
UserRequest.java
UserResponse.java
UserMapper.java
UserController.java
UserService.java
UserRepository.java
user-schema.sql
user-test-data.json
...
```

---

## Performance Testing Patterns

### Load Testing with K6

```javascript
// k6/order-creation.js
import http from 'k6/http';
import { check, sleep } from 'k6';
import { Rate, Trend } from 'k6/metrics';

const errorRate = new Rate('errors');
const orderCreationTime = new Trend('order_creation_time');

export const options = {
  stages: [
    { duration: '1m', target: 50 },   // Ramp up
    { duration: '5m', target: 50 },   // Steady state
    { duration: '1m', target: 100 },  // Spike
    { duration: '2m', target: 100 },  // Sustained spike
    { duration: '1m', target: 0 },    // Ramp down
  ],
  thresholds: {
    'http_req_duration': ['p(99)<500'],  // 99% under 500ms
    'errors': ['rate<0.01'],              // Error rate under 1%
  },
};

export default function () {
  const payload = JSON.stringify({
    customerId: `customer-${__VU}`,
    items: [
      { productId: 'product-1', quantity: Math.floor(Math.random() * 5) + 1 },
    ],
  });

  const params = {
    headers: {
      'Content-Type': 'application/json',
      'Authorization': `Bearer ${__ENV.API_TOKEN}`,
    },
  };

  const start = Date.now();
  const res = http.post(`${__ENV.API_URL}/api/orders`, payload, params);
  orderCreationTime.add(Date.now() - start);

  const success = check(res, {
    'status is 201': (r) => r.status === 201,
    'has order id': (r) => JSON.parse(r.body).id !== undefined,
  });

  errorRate.add(!success);
  sleep(1);
}
```

---

## Mermaid Diagrams (GitHub Compatible)

### System Architecture

```mermaid
flowchart TB
    subgraph Clients
        Web[Web App]
        Mobile[Mobile App]
        API[External API]
    end
    
    subgraph Gateway
        Kong[API Gateway]
        Auth[Auth Service]
    end
    
    subgraph Services
        Order[Order Service]
        Inventory[Inventory Service]
        Payment[Payment Service]
        Notification[Notification Service]
    end
    
    subgraph Data
        PostgresOrder[(PostgreSQL<br/>Orders)]
        PostgresInv[(PostgreSQL<br/>Inventory)]
        Redis[(Redis<br/>Cache)]
        Kafka[Apache Kafka]
    end
    
    Web --> Kong
    Mobile --> Kong
    API --> Kong
    
    Kong --> Auth
    Kong --> Order
    Kong --> Inventory
    Kong --> Payment
    
    Order --> PostgresOrder
    Order --> Kafka
    Inventory --> PostgresInv
    Inventory --> Redis
    Payment --> Kafka
    Notification --> Kafka
```

### Event Flow

```mermaid
sequenceDiagram
    participant C as Client
    participant O as Order Service
    participant I as Inventory Service
    participant P as Payment Service
    participant K as Kafka
    participant N as Notification
    
    C->>O: POST /orders
    O->>I: Check availability
    I-->>O: Available
    O->>I: Reserve inventory
    I-->>O: Reserved
    O->>O: Create order (PENDING)
    O->>K: Publish OrderCreated
    O-->>C: 201 Created
    
    K-->>P: OrderCreated
    P->>P: Process payment
    P->>K: Publish PaymentProcessed
    
    K-->>O: PaymentProcessed
    O->>O: Update order (CONFIRMED)
    O->>K: Publish OrderConfirmed
    
    K-->>N: OrderConfirmed
    N->>N: Send confirmation email
```

---

This documentation provides the TDD-first architecture principles needed to build production-grade systems. Follow the STUB → RED → GREEN → REFACTOR cycle, use executable specifications, and leverage the quality checklists to ensure robust, maintainable code.

# S77: Idiomatic Polyglot Patterns
## TypeScript/JavaScript • Java/Spring Boot • Golang • DevOps • Apache Kafka

========================================
SECTION 1: TYPESCRIPT/JAVASCRIPT PATTERNS
========================================

## TS.1 Branded Types for Domain IDs
- **Use when**: You need type-safe IDs that prevent mixing different entity types
- **Context**: Create nominal types using intersection with unique symbol
```typescript
// Define branded types
type UserId = string & { readonly __brand: unique symbol };
type OrderId = string & { readonly __brand: unique symbol };

// Constructor functions
const UserId = (id: string): UserId => id as UserId;
const OrderId = (id: string): OrderId => id as OrderId;

// Usage - compiler prevents mixing
function getUser(id: UserId): Promise<User> { /* ... */ }
function getOrder(id: OrderId): Promise<Order> { /* ... */ }

const userId = UserId('user-123');
const orderId = OrderId('order-456');

getUser(userId);  // ✅ OK
getUser(orderId); // ❌ Compile error - OrderId not assignable to UserId
```
- **Avoid/Anti-pattern**: Using plain `string` for all IDs
```typescript
// ❌ Bad - no type safety
function getUser(id: string): Promise<User> { }
function getOrder(id: string): Promise<Order> { }

getUser(orderId); // No error, runtime bug waiting to happen
```
- **Testing**:
```typescript
it('should reject invalid ID type at compile time', () => {
  const userId = UserId('u-1');
  // This test verifies the type system - actual compile error
  // @ts-expect-error - OrderId not assignable to UserId
  getUser(OrderId('o-1'));
});
```

---

## TS.2 Discriminated Unions for State Machines
- **Use when**: Modeling states with different associated data (loading, success, error)
- **Context**: Use literal type discriminant for exhaustive checking
```typescript
type AsyncState<T, E = Error> =
  | { status: 'idle' }
  | { status: 'loading' }
  | { status: 'success'; data: T }
  | { status: 'error'; error: E };

function renderUser(state: AsyncState<User>) {
  switch (state.status) {
    case 'idle':
      return <IdleView />;
    case 'loading':
      return <Spinner />;
    case 'success':
      return <UserCard user={state.data} />; // data is typed as User
    case 'error':
      return <ErrorMessage error={state.error} />;
    // No default needed - exhaustive
  }
}

// Type narrowing works automatically
if (state.status === 'success') {
  console.log(state.data.name); // TypeScript knows data exists
}
```
- **Avoid/Anti-pattern**: Separate boolean flags
```typescript
// ❌ Bad - impossible states representable
type BadState<T> = {
  isLoading: boolean;
  isError: boolean;
  data?: T;
  error?: Error;
};
// Can have isLoading: true AND isError: true - invalid!
```

---

## TS.3 Const Assertions for Literal Types
- **Use when**: You need exact literal types, not widened types
- **Context**: Use `as const` for objects and arrays to preserve literal types
```typescript
// Without const assertion
const config = {
  endpoint: '/api/users',
  method: 'GET',
}; // type: { endpoint: string; method: string }

// With const assertion
const config = {
  endpoint: '/api/users',
  method: 'GET',
} as const; // type: { readonly endpoint: '/api/users'; readonly method: 'GET' }

// Useful for action types
const ActionTypes = {
  ADD_USER: 'ADD_USER',
  REMOVE_USER: 'REMOVE_USER',
} as const;

type ActionType = typeof ActionTypes[keyof typeof ActionTypes];
// type: 'ADD_USER' | 'REMOVE_USER'
```
- **Avoid/Anti-pattern**: Manual readonly annotations everywhere
```typescript
// ❌ Verbose and error-prone
const config: { readonly endpoint: '/api/users'; readonly method: 'GET' } = {
  endpoint: '/api/users',
  method: 'GET',
};
```

---

## TS.4 Type Predicates for Custom Type Guards
- **Use when**: Filtering or checking types at runtime with type narrowing
- **Context**: Return `value is Type` for TypeScript to narrow types
```typescript
interface User {
  type: 'user';
  id: string;
  name: string;
}

interface Admin {
  type: 'admin';
  id: string;
  name: string;
  permissions: string[];
}

type Person = User | Admin;

// Type predicate
function isAdmin(person: Person): person is Admin {
  return person.type === 'admin';
}

// Usage with filtering
const people: Person[] = [/* ... */];
const admins: Admin[] = people.filter(isAdmin); // Correctly typed as Admin[]

// Usage in conditionals
function getPermissions(person: Person): string[] {
  if (isAdmin(person)) {
    return person.permissions; // TypeScript knows it's Admin
  }
  return [];
}
```
- **Avoid/Anti-pattern**: Returning boolean without type predicate
```typescript
// ❌ Bad - no type narrowing
function isAdmin(person: Person): boolean {
  return person.type === 'admin';
}

const admins = people.filter(isAdmin); // Still typed as Person[], not Admin[]
```

---

## TS.5 Generics with Constraints
- **Use when**: Writing reusable functions/classes that work with multiple types
- **Context**: Use `extends` to constrain generic types
```typescript
// Constrained generic
function getProperty<T, K extends keyof T>(obj: T, key: K): T[K] {
  return obj[key];
}

const user = { name: 'John', age: 30 };
const name = getProperty(user, 'name'); // type: string
const age = getProperty(user, 'age');   // type: number
// getProperty(user, 'invalid');        // ❌ Compile error

// Generic with multiple constraints
function merge<T extends object, U extends object>(a: T, b: U): T & U {
  return { ...a, ...b };
}

// Generic class with constraint
class Repository<T extends { id: string }> {
  private items: Map<string, T> = new Map();
  
  save(item: T): void {
    this.items.set(item.id, item);
  }
  
  findById(id: string): T | undefined {
    return this.items.get(id);
  }
}
```
- **Avoid/Anti-pattern**: Using `any` instead of generics
```typescript
// ❌ Bad - loses type safety
function getProperty(obj: any, key: string): any {
  return obj[key];
}
```

---

## TS.6 Template Literal Types
- **Use when**: Creating string types from combinations of other types
- **Context**: Use template literals for type-safe string patterns
```typescript
type HTTPMethod = 'GET' | 'POST' | 'PUT' | 'DELETE';
type APIVersion = 'v1' | 'v2';
type Resource = 'users' | 'orders' | 'products';

// Combine into endpoint pattern
type Endpoint = `/${APIVersion}/${Resource}`;
// type: '/v1/users' | '/v1/orders' | '/v1/products' | '/v2/users' | ...

// Event names
type EventName<T extends string> = `on${Capitalize<T>}`;
type ClickEvent = EventName<'click'>; // 'onClick'

// CSS properties
type CSSProperty = 'margin' | 'padding';
type CSSDirection = 'Top' | 'Right' | 'Bottom' | 'Left';
type CSSSpacing = `${CSSProperty}${CSSDirection}`;
// 'marginTop' | 'marginRight' | 'paddingTop' | ...

// Type-safe event handler
type EventHandler<T extends string> = {
  [K in `on${Capitalize<T>}`]: () => void;
};

const handlers: EventHandler<'click' | 'hover'> = {
  onClick: () => console.log('clicked'),
  onHover: () => console.log('hovered'),
};
```

---

## TS.7 Zod Runtime Validation with Type Inference
- **Use when**: Validating external data (API responses, form inputs) with type safety
- **Context**: Define schema once, get runtime validation AND TypeScript types
```typescript
import { z } from 'zod';

// Define schema
const UserSchema = z.object({
  id: z.string().uuid(),
  email: z.string().email(),
  name: z.string().min(1).max(100),
  age: z.number().int().min(0).max(150).optional(),
  role: z.enum(['user', 'admin', 'moderator']),
  createdAt: z.coerce.date(),
});

// Infer TypeScript type from schema
type User = z.infer<typeof UserSchema>;
// {
//   id: string;
//   email: string;
//   name: string;
//   age?: number;
//   role: 'user' | 'admin' | 'moderator';
//   createdAt: Date;
// }

// Runtime validation
function parseUser(data: unknown): User {
  return UserSchema.parse(data); // Throws ZodError if invalid
}

// Safe parsing (doesn't throw)
function safeParseUser(data: unknown): User | null {
  const result = UserSchema.safeParse(data);
  return result.success ? result.data : null;
}

// API response validation
async function fetchUser(id: string): Promise<User> {
  const response = await fetch(`/api/users/${id}`);
  const data = await response.json();
  return UserSchema.parse(data); // Validated and typed
}
```
- **Avoid/Anti-pattern**: Type assertions without validation
```typescript
// ❌ Bad - trusting external data
async function fetchUser(id: string): Promise<User> {
  const response = await fetch(`/api/users/${id}`);
  return await response.json() as User; // Could be anything!
}
```

---

## TS.8 Async/Await Error Handling Patterns
- **Use when**: Handling async operations with proper error management
- **Context**: Wrap in try/catch or use Result pattern
```typescript
// Pattern 1: Try-catch wrapper
async function tryCatch<T>(
  promise: Promise<T>
): Promise<[T, null] | [null, Error]> {
  try {
    const data = await promise;
    return [data, null];
  } catch (error) {
    return [null, error instanceof Error ? error : new Error(String(error))];
  }
}

// Usage
const [user, error] = await tryCatch(fetchUser(id));
if (error) {
  console.error('Failed to fetch user:', error.message);
  return;
}
console.log(user.name); // TypeScript knows user is not null

// Pattern 2: Result type for async operations
type AsyncResult<T, E = Error> = Promise<
  { ok: true; value: T } | { ok: false; error: E }
>;

async function fetchUserSafe(id: string): AsyncResult<User, APIError> {
  try {
    const response = await fetch(`/api/users/${id}`);
    if (!response.ok) {
      return { ok: false, error: { code: response.status, message: response.statusText } };
    }
    const data = await response.json();
    return { ok: true, value: UserSchema.parse(data) };
  } catch (e) {
    return { ok: false, error: { code: 500, message: 'Network error' } };
  }
}

// Pattern 3: Promise.allSettled for parallel operations
async function fetchAllUsers(ids: string[]): Promise<Map<string, User>> {
  const results = await Promise.allSettled(ids.map(fetchUser));
  const users = new Map<string, User>();
  
  results.forEach((result, index) => {
    if (result.status === 'fulfilled') {
      users.set(ids[index], result.value);
    } else {
      console.error(`Failed to fetch user ${ids[index]}:`, result.reason);
    }
  });
  
  return users;
}
```
- **Avoid/Anti-pattern**: Unhandled promise rejections
```typescript
// ❌ Bad - unhandled rejection
async function handleClick() {
  const user = await fetchUser(id); // If this throws, error is swallowed
  updateUI(user);
}
```

---

## TS.9 Custom Hooks with Proper TypeScript
- **Use when**: Extracting reusable stateful logic in React
- **Context**: Define return type explicitly for complex hooks
```typescript
// Hook with proper types
interface UseAsyncOptions<T> {
  immediate?: boolean;
  onSuccess?: (data: T) => void;
  onError?: (error: Error) => void;
}

interface UseAsyncReturn<T, Args extends unknown[]> {
  data: T | null;
  error: Error | null;
  isLoading: boolean;
  execute: (...args: Args) => Promise<T | null>;
  reset: () => void;
}

function useAsync<T, Args extends unknown[] = []>(
  asyncFn: (...args: Args) => Promise<T>,
  options: UseAsyncOptions<T> = {}
): UseAsyncReturn<T, Args> {
  const [data, setData] = useState<T | null>(null);
  const [error, setError] = useState<Error | null>(null);
  const [isLoading, setIsLoading] = useState(false);

  const execute = useCallback(async (...args: Args): Promise<T | null> => {
    setIsLoading(true);
    setError(null);
    try {
      const result = await asyncFn(...args);
      setData(result);
      options.onSuccess?.(result);
      return result;
    } catch (e) {
      const err = e instanceof Error ? e : new Error(String(e));
      setError(err);
      options.onError?.(err);
      return null;
    } finally {
      setIsLoading(false);
    }
  }, [asyncFn, options.onSuccess, options.onError]);

  const reset = useCallback(() => {
    setData(null);
    setError(null);
    setIsLoading(false);
  }, []);

  return { data, error, isLoading, execute, reset };
}

// Usage
function UserProfile({ userId }: { userId: string }) {
  const { data: user, isLoading, error, execute } = useAsync(
    (id: string) => fetchUser(id),
    { onError: (e) => toast.error(e.message) }
  );

  useEffect(() => {
    execute(userId);
  }, [userId, execute]);

  if (isLoading) return <Spinner />;
  if (error) return <ErrorMessage error={error} />;
  if (!user) return null;
  
  return <UserCard user={user} />;
}
```

---

## TS.10 Module Augmentation for Third-Party Types
- **Use when**: Extending types from libraries without modifying source
- **Context**: Use declaration merging to add properties
```typescript
// Extend Express Request type
declare global {
  namespace Express {
    interface Request {
      user?: {
        id: string;
        email: string;
        roles: string[];
      };
      requestId: string;
    }
  }
}

// Now TypeScript knows about req.user and req.requestId
app.get('/profile', authenticate, (req, res) => {
  const userId = req.user?.id; // Typed correctly
  const requestId = req.requestId;
  // ...
});

// Extend Window for global variables
declare global {
  interface Window {
    __CONFIG__: {
      apiUrl: string;
      environment: 'development' | 'staging' | 'production';
    };
  }
}

// Now window.__CONFIG__ is typed
const apiUrl = window.__CONFIG__.apiUrl;

// Extend existing module
declare module 'axios' {
  export interface AxiosRequestConfig {
    retry?: number;
    retryDelay?: number;
  }
}
```

---

## TS.11 Mapped Types for Transformations
- **Use when**: Creating new types by transforming existing ones
- **Context**: Use mapped types with modifiers
```typescript
// Make all properties optional
type Partial<T> = { [K in keyof T]?: T[K] };

// Make all properties required
type Required<T> = { [K in keyof T]-?: T[K] };

// Make all properties readonly
type Readonly<T> = { readonly [K in keyof T]: T[K] };

// Pick specific keys
type Pick<T, K extends keyof T> = { [P in K]: T[P] };

// Custom: Make specific keys required
type RequireKeys<T, K extends keyof T> = T & Required<Pick<T, K>>;

interface User {
  id: string;
  name?: string;
  email?: string;
}

type UserWithEmail = RequireKeys<User, 'email'>;
// { id: string; name?: string; email: string }

// Custom: Nullable properties
type Nullable<T> = { [K in keyof T]: T[K] | null };

// Custom: Deep partial
type DeepPartial<T> = {
  [K in keyof T]?: T[K] extends object ? DeepPartial<T[K]> : T[K];
};

// Custom: Form state from entity
type FormState<T> = {
  values: T;
  errors: { [K in keyof T]?: string };
  touched: { [K in keyof T]?: boolean };
};
```

---

## TS.12 Infer Keyword in Conditional Types
- **Use when**: Extracting types from generic types
- **Context**: Use `infer` to capture type variables
```typescript
// Extract return type of function
type ReturnType<T> = T extends (...args: any[]) => infer R ? R : never;

// Extract element type of array
type ElementType<T> = T extends (infer E)[] ? E : never;

// Extract promise value type
type Awaited<T> = T extends Promise<infer V> ? Awaited<V> : T;

// Extract props type from React component
type PropsOf<T> = T extends React.ComponentType<infer P> ? P : never;

// Extract second parameter type
type SecondParam<T> = T extends (a: any, b: infer B, ...args: any[]) => any ? B : never;

// Practical example: Extract API response types
type APIResponse<T> = {
  data: T;
  status: number;
  message: string;
};

type ExtractData<T> = T extends APIResponse<infer D> ? D : never;

type UserResponse = APIResponse<User>;
type UserData = ExtractData<UserResponse>; // User

// Extract handler types from event map
type EventMap = {
  click: MouseEvent;
  keydown: KeyboardEvent;
  submit: FormEvent;
};

type EventHandler<K extends keyof EventMap> = (event: EventMap[K]) => void;
```

---

## TS.13 Function Overloads for API Design
- **Use when**: Function has different return types based on input
- **Context**: Define overload signatures before implementation
```typescript
// Overloaded function
function createElement(tag: 'input'): HTMLInputElement;
function createElement(tag: 'div'): HTMLDivElement;
function createElement(tag: 'span'): HTMLSpanElement;
function createElement(tag: string): HTMLElement;
function createElement(tag: string): HTMLElement {
  return document.createElement(tag);
}

const input = createElement('input'); // HTMLInputElement
const div = createElement('div');     // HTMLDivElement

// Overloaded with different param counts
function fetch(url: string): Promise<Response>;
function fetch(url: string, options: RequestInit): Promise<Response>;
function fetch(url: string, options?: RequestInit): Promise<Response> {
  return window.fetch(url, options);
}

// Overloaded method in class
class EventEmitter<Events extends Record<string, unknown[]>> {
  on<K extends keyof Events>(event: K, handler: (...args: Events[K]) => void): void;
  on(event: string, handler: (...args: unknown[]) => void): void {
    // Implementation
  }

  emit<K extends keyof Events>(event: K, ...args: Events[K]): void;
  emit(event: string, ...args: unknown[]): void {
    // Implementation
  }
}

// Usage
type AppEvents = {
  userLogin: [userId: string];
  orderPlaced: [orderId: string, total: number];
};

const emitter = new EventEmitter<AppEvents>();
emitter.on('userLogin', (userId) => { /* userId is string */ });
emitter.on('orderPlaced', (orderId, total) => { /* correctly typed */ });
```

---

## TS.14 Satisfies Operator for Type Checking
- **Use when**: Validate type compatibility while preserving literal types
- **Context**: Use `satisfies` instead of type annotation when you need both
```typescript
// Without satisfies - loses literal types
const routes: Record<string, string> = {
  home: '/',
  users: '/users',
  settings: '/settings',
};
// routes.home is string, not '/'

// With satisfies - keeps literal types
const routes = {
  home: '/',
  users: '/users',
  settings: '/settings',
} satisfies Record<string, string>;
// routes.home is '/' (literal type preserved)
// Still validated against Record<string, string>

// Practical use: config validation
type ColorConfig = Record<string, [number, number, number]>;

const colors = {
  red: [255, 0, 0],
  green: [0, 255, 0],
  blue: [0, 0, 255],
} satisfies ColorConfig;

// colors.red is [number, number, number] but also ensures validity
// Typo in color name? Caught at compile time
// Wrong tuple length? Caught at compile time

// With theme config
type Theme = {
  colors: Record<string, string>;
  spacing: Record<string, number>;
};

const theme = {
  colors: {
    primary: '#007bff',
    secondary: '#6c757d',
  },
  spacing: {
    sm: 8,
    md: 16,
    lg: 24,
  },
} satisfies Theme;

// theme.colors.primary is '#007bff' (literal preserved)
// theme.spacing.sm is 8 (literal preserved)
```

---

## TS.15 Testing Patterns with Vitest/Jest
- **Use when**: Writing type-safe tests with modern tooling
- **Context**: Use proper typing for mocks and assertions
```typescript
import { describe, it, expect, vi, beforeEach, type Mock } from 'vitest';

// Typed mock
const mockFetch = vi.fn<[string], Promise<User>>();

// Mock module with types
vi.mock('./api', () => ({
  fetchUser: vi.fn<[string], Promise<User>>(),
}));

import { fetchUser } from './api';

describe('UserService', () => {
  let service: UserService;
  
  beforeEach(() => {
    vi.clearAllMocks();
    service = new UserService(fetchUser as Mock);
  });

  it('should return user when found', async () => {
    const mockUser: User = { id: '1', name: 'Test', email: 'test@test.com' };
    vi.mocked(fetchUser).mockResolvedValue(mockUser);

    const result = await service.getUser('1');

    expect(result).toEqual(mockUser);
    expect(fetchUser).toHaveBeenCalledWith('1');
    expect(fetchUser).toHaveBeenCalledTimes(1);
  });

  it('should throw when user not found', async () => {
    vi.mocked(fetchUser).mockRejectedValue(new Error('Not found'));

    await expect(service.getUser('invalid')).rejects.toThrow('Not found');
  });
});

// Testing React components
import { render, screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';

describe('LoginForm', () => {
  it('should submit with valid credentials', async () => {
    const onSubmit = vi.fn<[{ email: string; password: string }], void>();
    const user = userEvent.setup();

    render(<LoginForm onSubmit={onSubmit} />);

    await user.type(screen.getByLabelText(/email/i), 'test@example.com');
    await user.type(screen.getByLabelText(/password/i), 'password123');
    await user.click(screen.getByRole('button', { name: /sign in/i }));

    await waitFor(() => {
      expect(onSubmit).toHaveBeenCalledWith({
        email: 'test@example.com',
        password: 'password123',
      });
    });
  });
});
```

---

========================================
SECTION 2: JAVA/SPRING BOOT PATTERNS
========================================

## J.1 Records for Immutable Data
- **Use when**: Creating DTOs, value objects, or immutable data carriers
- **Context**: Use records instead of classes with getters/setters
```java
// ✅ Good: Record for DTO
public record CreateUserRequest(
    @NotBlank String email,
    @NotBlank @Size(min = 2, max = 100) String name,
    @NotNull Role role
) {
    // Compact constructor for validation
    public CreateUserRequest {
        email = email.toLowerCase().trim();
        name = name.trim();
    }
}

// ✅ Good: Record for value object
public record Money(BigDecimal amount, Currency currency) {
    public Money {
        Objects.requireNonNull(amount, "amount must not be null");
        Objects.requireNonNull(currency, "currency must not be null");
        if (amount.compareTo(BigDecimal.ZERO) < 0) {
            throw new IllegalArgumentException("amount must not be negative");
        }
    }
    
    public Money add(Money other) {
        if (!this.currency.equals(other.currency)) {
            throw new IllegalArgumentException("Cannot add different currencies");
        }
        return new Money(this.amount.add(other.amount), this.currency);
    }
}

// ✅ Good: Record for query results
public record UserSummary(String id, String name, long orderCount) {}
```
- **Avoid/Anti-pattern**: Mutable POJOs with setters
```java
// ❌ Bad: Mutable, verbose
public class CreateUserRequest {
    private String email;
    private String name;
    // 50 lines of getters, setters, equals, hashCode, toString...
}
```

---

## J.2 Sealed Classes for Domain Modeling
- **Use when**: Modeling closed hierarchies with exhaustive pattern matching
- **Context**: Use sealed classes with permits clause
```java
// Domain events as sealed hierarchy
public sealed interface OrderEvent permits 
    OrderCreated, OrderConfirmed, OrderShipped, OrderDelivered, OrderCancelled {
    
    String orderId();
    Instant occurredAt();
}

public record OrderCreated(String orderId, Instant occurredAt, List<OrderItem> items) 
    implements OrderEvent {}

public record OrderShipped(String orderId, Instant occurredAt, String trackingNumber) 
    implements OrderEvent {}

public record OrderCancelled(String orderId, Instant occurredAt, String reason) 
    implements OrderEvent {}

// Pattern matching (Java 21+)
public String handleEvent(OrderEvent event) {
    return switch (event) {
        case OrderCreated e -> "Order %s created with %d items".formatted(e.orderId(), e.items().size());
        case OrderConfirmed e -> "Order %s confirmed".formatted(e.orderId());
        case OrderShipped e -> "Order %s shipped, tracking: %s".formatted(e.orderId(), e.trackingNumber());
        case OrderDelivered e -> "Order %s delivered".formatted(e.orderId());
        case OrderCancelled e -> "Order %s cancelled: %s".formatted(e.orderId(), e.reason());
        // No default needed - compiler ensures exhaustiveness
    };
}

// Result type with sealed classes
public sealed interface Result<T> permits Result.Success, Result.Failure {
    record Success<T>(T value) implements Result<T> {}
    record Failure<T>(String error, String code) implements Result<T> {}
    
    static <T> Result<T> success(T value) { return new Success<>(value); }
    static <T> Result<T> failure(String error, String code) { return new Failure<>(error, code); }
    
    default T getOrThrow() {
        return switch (this) {
            case Success<T> s -> s.value();
            case Failure<T> f -> throw new RuntimeException(f.error());
        };
    }
}
```

---

## J.3 Optional for Nullable Returns
- **Use when**: Method might not return a value
- **Context**: Use Optional for return types, NEVER for parameters or fields
```java
public interface UserRepository {
    // ✅ Good: Optional for nullable return
    Optional<User> findById(String id);
    
    Optional<User> findByEmail(String email);
    
    // ✅ Good: Collection for multiple results (empty, not null)
    List<User> findByRole(Role role);
    
    // ❌ Bad: Never use Optional for parameters
    // void updateEmail(String id, Optional<String> newEmail);
    
    // ✅ Good: Use overloading or null instead
    void updateEmail(String id, String newEmail);
}

// Using Optional effectively
public class UserService {
    private final UserRepository repository;
    
    public UserDTO getUser(String id) {
        return repository.findById(id)
            .map(this::toDTO)
            .orElseThrow(() -> new UserNotFoundException(id));
    }
    
    public UserDTO getUserOrDefault(String id) {
        return repository.findById(id)
            .map(this::toDTO)
            .orElseGet(UserDTO::anonymous);
    }
    
    // Chaining Optionals
    public Optional<String> getUserEmail(String userId) {
        return repository.findById(userId)
            .map(User::email)
            .filter(email -> !email.isBlank());
    }
    
    // flatMap for nested Optionals
    public Optional<Address> getUserAddress(String userId) {
        return repository.findById(userId)
            .flatMap(User::address); // User::address returns Optional<Address>
    }
}
```
- **Avoid/Anti-pattern**: Optional.get() without checking
```java
// ❌ Bad - defeats the purpose
Optional<User> userOpt = repository.findById(id);
User user = userOpt.get(); // NoSuchElementException if empty

// ❌ Bad - verbose null check anyway
if (userOpt.isPresent()) {
    User user = userOpt.get();
}

// ✅ Good - use map/orElse/orElseThrow
User user = userOpt.orElseThrow(() -> new UserNotFoundException(id));
```

---

## J.4 Spring Dependency Injection Best Practices
- **Use when**: Wiring Spring beans together
- **Context**: Use constructor injection exclusively
```java
// ✅ Good: Constructor injection (preferred)
@Service
public class UserService {
    private final UserRepository repository;
    private final EventPublisher events;
    private final UserMapper mapper;
    
    // Single constructor - @Autowired optional
    public UserService(UserRepository repository, 
                       EventPublisher events, 
                       UserMapper mapper) {
        this.repository = repository;
        this.events = events;
        this.mapper = mapper;
    }
}

// ✅ Good: With Lombok
@Service
@RequiredArgsConstructor
public class UserService {
    private final UserRepository repository;
    private final EventPublisher events;
    private final UserMapper mapper;
}

// ✅ Good: Interface segregation
public interface UserReader {
    Optional<User> findById(String id);
    List<User> findAll();
}

public interface UserWriter {
    User save(User user);
    void delete(String id);
}

@Service
@RequiredArgsConstructor
public class UserQueryService {
    private final UserReader reader; // Only read operations
}

@Service
@RequiredArgsConstructor
public class UserCommandService {
    private final UserWriter writer; // Only write operations
}
```
- **Avoid/Anti-pattern**: Field injection
```java
// ❌ Bad: Field injection - hidden dependencies, untestable
@Service
public class UserService {
    @Autowired
    private UserRepository repository;
    
    @Autowired
    private EventPublisher events;
    
    // Cannot test without Spring context
    // Cannot see dependencies at construction time
}
```

---

## J.5 Spring Profiles and Configuration
- **Use when**: Managing environment-specific configuration
- **Context**: Use profiles for environment separation, @ConfigurationProperties for type safety
```java
// Type-safe configuration
@ConfigurationProperties(prefix = "app.database")
@Validated
public record DatabaseProperties(
    @NotBlank String url,
    @NotBlank String username,
    @NotBlank String password,
    @Min(1) @Max(100) int poolSize,
    @DurationMin(seconds = 1) Duration connectionTimeout
) {}

// Enable configuration properties
@SpringBootApplication
@EnableConfigurationProperties(DatabaseProperties.class)
public class Application {
    public static void main(String[] args) {
        SpringApplication.run(Application.class, args);
    }
}

// application.yml
# app:
#   database:
#     url: jdbc:postgresql://localhost:5432/mydb
#     username: user
#     password: ${DB_PASSWORD}
#     pool-size: 10
#     connection-timeout: 30s

// Profile-specific configuration
@Configuration
@Profile("!test") // Not in test profile
public class KafkaProducerConfig {
    @Bean
    public KafkaTemplate<String, Object> kafkaTemplate(KafkaProperties props) {
        return new KafkaTemplate<>(new DefaultKafkaProducerFactory<>(
            props.buildProducerProperties()
        ));
    }
}

@Configuration
@Profile("test")
public class TestKafkaConfig {
    @Bean
    public KafkaTemplate<String, Object> kafkaTemplate() {
        return Mockito.mock(KafkaTemplate.class);
    }
}

// Conditional beans
@Bean
@ConditionalOnProperty(name = "app.feature.new-algorithm", havingValue = "true")
public PricingService newPricingService() {
    return new NewPricingService();
}

@Bean
@ConditionalOnMissingBean(PricingService.class)
public PricingService defaultPricingService() {
    return new DefaultPricingService();
}
```

---

## J.6 JPA Entity Best Practices
- **Use when**: Mapping domain objects to database
- **Context**: Proper equals/hashCode, lazy loading, bidirectional relationships
```java
@Entity
@Table(name = "users")
public class User {
    @Id
    @GeneratedValue(strategy = GenerationType.UUID)
    private String id;
    
    @Column(nullable = false, unique = true)
    private String email;
    
    @Column(nullable = false)
    private String name;
    
    @Enumerated(EnumType.STRING)
    @Column(nullable = false)
    private UserStatus status;
    
    @OneToMany(mappedBy = "user", cascade = CascadeType.ALL, orphanRemoval = true)
    private List<Order> orders = new ArrayList<>();
    
    @Version
    private Long version; // Optimistic locking
    
    @CreationTimestamp
    private Instant createdAt;
    
    @UpdateTimestamp
    private Instant updatedAt;
    
    // Protected constructor for JPA
    protected User() {}
    
    // Factory method
    public static User create(String email, String name) {
        User user = new User();
        user.email = email.toLowerCase();
        user.name = name;
        user.status = UserStatus.ACTIVE;
        return user;
    }
    
    // Bidirectional relationship management
    public void addOrder(Order order) {
        orders.add(order);
        order.setUser(this);
    }
    
    public void removeOrder(Order order) {
        orders.remove(order);
        order.setUser(null);
    }
    
    // Equals/hashCode based on business key or ID
    @Override
    public boolean equals(Object o) {
        if (this == o) return true;
        if (!(o instanceof User user)) return false;
        return id != null && id.equals(user.id);
    }
    
    @Override
    public int hashCode() {
        return getClass().hashCode(); // Consistent across entity state transitions
    }
}
```
- **Avoid/Anti-pattern**: N+1 queries
```java
// ❌ Bad: N+1 problem
List<User> users = userRepository.findAll(); // 1 query
for (User user : users) {
    user.getOrders().size(); // N additional queries!
}

// ✅ Good: Fetch join
@Query("SELECT u FROM User u LEFT JOIN FETCH u.orders WHERE u.status = :status")
List<User> findByStatusWithOrders(@Param("status") UserStatus status);

// ✅ Good: Entity graph
@EntityGraph(attributePaths = {"orders", "orders.items"})
List<User> findByStatus(UserStatus status);
```

---

## J.7 Spring WebFlux Reactive Patterns
- **Use when**: Building non-blocking, reactive web applications
- **Context**: Use Mono for 0-1 results, Flux for 0-N results
```java
@RestController
@RequestMapping("/api/users")
@RequiredArgsConstructor
public class UserController {
    private final UserService userService;
    
    // Mono for single result
    @GetMapping("/{id}")
    public Mono<ResponseEntity<UserDTO>> getUser(@PathVariable String id) {
        return userService.findById(id)
            .map(ResponseEntity::ok)
            .defaultIfEmpty(ResponseEntity.notFound().build());
    }
    
    // Flux for multiple results
    @GetMapping
    public Flux<UserDTO> getAllUsers() {
        return userService.findAll();
    }
    
    // Streaming response
    @GetMapping(value = "/stream", produces = MediaType.TEXT_EVENT_STREAM_VALUE)
    public Flux<UserDTO> streamUsers() {
        return userService.findAll()
            .delayElements(Duration.ofMillis(100)); // Simulate streaming
    }
    
    // Error handling
    @PostMapping
    public Mono<ResponseEntity<UserDTO>> createUser(@Valid @RequestBody CreateUserRequest request) {
        return userService.create(request)
            .map(user -> ResponseEntity.created(URI.create("/api/users/" + user.id())).body(user))
            .onErrorResume(DuplicateEmailException.class, e -> 
                Mono.just(ResponseEntity.status(HttpStatus.CONFLICT).build()));
    }
}

@Service
@RequiredArgsConstructor
public class UserService {
    private final UserRepository repository;
    private final EventPublisher events;
    
    public Mono<UserDTO> findById(String id) {
        return repository.findById(id)
            .map(UserDTO::from);
    }
    
    public Flux<UserDTO> findAll() {
        return repository.findAll()
            .map(UserDTO::from);
    }
    
    public Mono<UserDTO> create(CreateUserRequest request) {
        return repository.existsByEmail(request.email())
            .flatMap(exists -> {
                if (exists) {
                    return Mono.error(new DuplicateEmailException(request.email()));
                }
                return repository.save(User.from(request));
            })
            .doOnSuccess(user -> events.publish(new UserCreatedEvent(user)))
            .map(UserDTO::from);
    }
    
    // Combining multiple reactive streams
    public Mono<UserProfileDTO> getProfile(String userId) {
        return Mono.zip(
            repository.findById(userId),
            orderRepository.countByUserId(userId),
            reviewRepository.getAverageRating(userId)
        ).map(tuple -> new UserProfileDTO(
            tuple.getT1(),
            tuple.getT2(),
            tuple.getT3()
        ));
    }
}
```

---

## J.8 Testing with JUnit 5 and Spring Boot Test
- **Use when**: Testing Spring applications at various levels
- **Context**: Use appropriate test slices for focused testing
```java
// Unit test - no Spring context
class UserServiceTest {
    @Mock UserRepository repository;
    @Mock EventPublisher events;
    @InjectMocks UserServiceImpl service;
    
    @BeforeEach
    void setUp() {
        MockitoAnnotations.openMocks(this);
    }
    
    @Test
    void shouldCreateUserWithValidData() {
        // Given
        var request = new CreateUserRequest("test@example.com", "Test User", Role.USER);
        when(repository.existsByEmail(request.email())).thenReturn(false);
        when(repository.save(any())).thenAnswer(inv -> inv.getArgument(0));
        
        // When
        var result = service.createUser(request);
        
        // Then
        assertThat(result.isSuccess()).isTrue();
        assertThat(result.getValue().email()).isEqualTo("test@example.com");
        verify(events).publish(any(UserCreatedEvent.class));
    }
    
    @ParameterizedTest
    @CsvSource({
        "admin@test.com, Admin, ADMIN",
        "user@test.com, User, USER"
    })
    void shouldCreateUserWithDifferentRoles(String email, String name, Role role) {
        var request = new CreateUserRequest(email, name, role);
        when(repository.existsByEmail(any())).thenReturn(false);
        when(repository.save(any())).thenAnswer(inv -> inv.getArgument(0));
        
        var result = service.createUser(request);
        
        assertThat(result.getValue().role()).isEqualTo(role);
    }
}

// Integration test with TestContainers
@SpringBootTest
@Testcontainers
class UserRepositoryIT {
    @Container
    static PostgreSQLContainer<?> postgres = new PostgreSQLContainer<>("postgres:15")
        .withDatabaseName("test")
        .withUsername("test")
        .withPassword("test");
    
    @DynamicPropertySource
    static void configureProperties(DynamicPropertyRegistry registry) {
        registry.add("spring.datasource.url", postgres::getJdbcUrl);
        registry.add("spring.datasource.username", postgres::getUsername);
        registry.add("spring.datasource.password", postgres::getPassword);
    }
    
    @Autowired UserRepository repository;
    
    @Test
    void shouldPersistAndRetrieveUser() {
        var user = User.create("test@example.com", "Test User");
        
        var saved = repository.save(user);
        var found = repository.findById(saved.getId());
        
        assertThat(found).isPresent();
        assertThat(found.get().getEmail()).isEqualTo("test@example.com");
    }
}

// Controller test with MockMvc
@WebMvcTest(UserController.class)
class UserControllerTest {
    @Autowired MockMvc mockMvc;
    @MockBean UserService userService;
    @Autowired ObjectMapper objectMapper;
    
    @Test
    void shouldReturnUserWhenFound() throws Exception {
        var user = new UserDTO("1", "test@example.com", "Test");
        when(userService.findById("1")).thenReturn(Optional.of(user));
        
        mockMvc.perform(get("/api/users/1"))
            .andExpect(status().isOk())
            .andExpect(jsonPath("$.email").value("test@example.com"));
    }
    
    @Test
    void shouldReturn404WhenNotFound() throws Exception {
        when(userService.findById("999")).thenReturn(Optional.empty());
        
        mockMvc.perform(get("/api/users/999"))
            .andExpect(status().isNotFound());
    }
    
    @Test
    void shouldValidateRequestBody() throws Exception {
        var invalidRequest = new CreateUserRequest("", "", null);
        
        mockMvc.perform(post("/api/users")
                .contentType(MediaType.APPLICATION_JSON)
                .content(objectMapper.writeValueAsString(invalidRequest)))
            .andExpect(status().isBadRequest());
    }
}
```

---

## J.9 Virtual Threads (Java 21+)
- **Use when**: High-throughput I/O-bound applications
- **Context**: Use virtual threads for blocking operations without platform thread exhaustion
```java
// Enable virtual threads in Spring Boot 3.2+
# application.yml
# spring:
#   threads:
#     virtual:
#       enabled: true

// Manual virtual thread usage
public class VirtualThreadExample {
    
    public List<UserDTO> fetchAllUsersParallel(List<String> userIds) {
        try (var executor = Executors.newVirtualThreadPerTaskExecutor()) {
            List<Future<UserDTO>> futures = userIds.stream()
                .map(id -> executor.submit(() -> fetchUser(id)))
                .toList();
            
            return futures.stream()
                .map(this::getFutureResult)
                .filter(Objects::nonNull)
                .toList();
        }
    }
    
    private UserDTO getFutureResult(Future<UserDTO> future) {
        try {
            return future.get(5, TimeUnit.SECONDS);
        } catch (Exception e) {
            log.error("Failed to fetch user", e);
            return null;
        }
    }
    
    // Structured concurrency (Java 21 Preview)
    public UserProfile fetchUserProfile(String userId) throws Exception {
        try (var scope = new StructuredTaskScope.ShutdownOnFailure()) {
            Subtask<User> userTask = scope.fork(() -> userRepository.findById(userId).orElseThrow());
            Subtask<List<Order>> ordersTask = scope.fork(() -> orderRepository.findByUserId(userId));
            Subtask<Double> ratingTask = scope.fork(() -> reviewService.getAverageRating(userId));
            
            scope.join();
            scope.throwIfFailed();
            
            return new UserProfile(
                userTask.get(),
                ordersTask.get(),
                ratingTask.get()
            );
        }
    }
}

// Virtual thread-safe patterns
@Service
public class UserService {
    // ✅ Thread-safe: final fields, immutable state
    private final UserRepository repository;
    private final LoadingCache<String, UserDTO> cache;
    
    public UserService(UserRepository repository) {
        this.repository = repository;
        this.cache = Caffeine.newBuilder()
            .maximumSize(10_000)
            .expireAfterWrite(Duration.ofMinutes(5))
            .build(this::loadUser);
    }
    
    public UserDTO getUser(String id) {
        return cache.get(id); // Thread-safe
    }
}
```

---

## J.10 Streams API Best Practices
- **Use when**: Processing collections declaratively
- **Context**: Use streams for transformation, filtering, aggregation
```java
public class StreamPatterns {
    
    // Basic transformations
    public List<UserDTO> toUserDTOs(List<User> users) {
        return users.stream()
            .filter(User::isActive)
            .map(UserDTO::from)
            .sorted(Comparator.comparing(UserDTO::name))
            .toList(); // Java 16+ (prefer over collect(Collectors.toList()))
    }
    
    // Grouping
    public Map<Role, List<User>> groupByRole(List<User> users) {
        return users.stream()
            .collect(Collectors.groupingBy(User::role));
    }
    
    // Grouping with downstream collector
    public Map<Role, Long> countByRole(List<User> users) {
        return users.stream()
            .collect(Collectors.groupingBy(User::role, Collectors.counting()));
    }
    
    // Partitioning
    public Map<Boolean, List<User>> partitionByActive(List<User> users) {
        return users.stream()
            .collect(Collectors.partitioningBy(User::isActive));
    }
    
    // FlatMap for nested collections
    public List<OrderItem> getAllOrderItems(List<Order> orders) {
        return orders.stream()
            .flatMap(order -> order.items().stream())
            .toList();
    }
    
    // Reduce for aggregation
    public Money calculateTotal(List<OrderItem> items) {
        return items.stream()
            .map(item -> item.price().multiply(item.quantity()))
            .reduce(Money.ZERO, Money::add);
    }
    
    // Optional with streams
    public Optional<User> findHighestSpender(List<User> users) {
        return users.stream()
            .max(Comparator.comparing(User::totalSpent));
    }
    
    // Parallel streams (use carefully)
    public long countActiveUsers(List<User> users) {
        if (users.size() < 10_000) {
            return users.stream().filter(User::isActive).count();
        }
        return users.parallelStream().filter(User::isActive).count();
    }
    
    // Collectors.teeing (Java 12+)
    public record Statistics(long count, double average) {}
    
    public Statistics calculateStatistics(List<Order> orders) {
        return orders.stream()
            .map(Order::total)
            .collect(Collectors.teeing(
                Collectors.counting(),
                Collectors.averagingDouble(BigDecimal::doubleValue),
                Statistics::new
            ));
    }
}
```
- **Avoid/Anti-pattern**: Side effects in streams
```java
// ❌ Bad: Side effects
List<User> activeUsers = new ArrayList<>();
users.stream()
    .filter(User::isActive)
    .forEach(activeUsers::add); // Side effect!

// ✅ Good: Pure transformation
List<User> activeUsers = users.stream()
    .filter(User::isActive)
    .toList();
```

---

========================================
SECTION 3: GOLANG PATTERNS
========================================

## G.1 Error Handling with Wrapping
- **Use when**: Propagating errors with context
- **Context**: Use fmt.Errorf with %w verb for error wrapping
```go
import (
    "errors"
    "fmt"
)

// Define sentinel errors
var (
    ErrNotFound      = errors.New("not found")
    ErrUnauthorized  = errors.New("unauthorized")
    ErrInvalidInput  = errors.New("invalid input")
)

// Custom error type
type ValidationError struct {
    Field   string
    Message string
}

func (e *ValidationError) Error() string {
    return fmt.Sprintf("validation error on field %s: %s", e.Field, e.Message)
}

// Error wrapping
func (s *UserService) GetUser(ctx context.Context, id string) (*User, error) {
    user, err := s.repo.FindByID(ctx, id)
    if err != nil {
        if errors.Is(err, sql.ErrNoRows) {
            return nil, fmt.Errorf("user %s: %w", id, ErrNotFound)
        }
        return nil, fmt.Errorf("fetching user %s: %w", id, err)
    }
    return user, nil
}

// Error checking
func handleUser(ctx context.Context, id string) error {
    user, err := userService.GetUser(ctx, id)
    if err != nil {
        if errors.Is(err, ErrNotFound) {
            // Handle not found
            return respondWithNotFound()
        }
        // Log and return internal error
        log.Error("failed to get user", "error", err, "userId", id)
        return respondWithInternalError()
    }
    return respond(user)
}

// Extracting wrapped errors
var validationErr *ValidationError
if errors.As(err, &validationErr) {
    return respondWithBadRequest(validationErr.Field, validationErr.Message)
}
```
- **Avoid/Anti-pattern**: Ignoring errors or using panic
```go
// ❌ Bad: Ignoring error
user, _ := userService.GetUser(ctx, id)

// ❌ Bad: Using panic for recoverable errors
func GetUser(id string) *User {
    user, err := repo.FindByID(id)
    if err != nil {
        panic(err) // Don't do this!
    }
    return user
}
```

---

## G.2 Context for Cancellation and Timeouts
- **Use when**: Managing request lifecycle, cancellation, timeouts
- **Context**: Pass context as first parameter, use WithTimeout/WithCancel
```go
import (
    "context"
    "time"
)

// Accept context as first parameter
func (s *UserService) GetUser(ctx context.Context, id string) (*User, error) {
    // Check if already cancelled
    if err := ctx.Err(); err != nil {
        return nil, fmt.Errorf("context cancelled: %w", err)
    }
    
    // Use context in database query
    user, err := s.db.QueryRowContext(ctx, 
        "SELECT id, name, email FROM users WHERE id = $1", id).Scan(&user)
    if err != nil {
        return nil, err
    }
    return user, nil
}

// Creating contexts with timeout
func (h *Handler) HandleRequest(w http.ResponseWriter, r *http.Request) {
    // Create context with timeout
    ctx, cancel := context.WithTimeout(r.Context(), 5*time.Second)
    defer cancel() // Always call cancel to release resources
    
    user, err := h.userService.GetUser(ctx, r.PathValue("id"))
    if err != nil {
        if errors.Is(err, context.DeadlineExceeded) {
            http.Error(w, "request timeout", http.StatusGatewayTimeout)
            return
        }
        http.Error(w, err.Error(), http.StatusInternalServerError)
        return
    }
    
    json.NewEncoder(w).Encode(user)
}

// Context values (use sparingly)
type contextKey string

const (
    requestIDKey contextKey = "requestID"
    userIDKey    contextKey = "userID"
)

func WithRequestID(ctx context.Context, requestID string) context.Context {
    return context.WithValue(ctx, requestIDKey, requestID)
}

func RequestIDFrom(ctx context.Context) string {
    if id, ok := ctx.Value(requestIDKey).(string); ok {
        return id
    }
    return ""
}

// Middleware adding context values
func RequestIDMiddleware(next http.Handler) http.Handler {
    return http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
        requestID := r.Header.Get("X-Request-ID")
        if requestID == "" {
            requestID = uuid.New().String()
        }
        ctx := WithRequestID(r.Context(), requestID)
        next.ServeHTTP(w, r.WithContext(ctx))
    })
}
```
- **Avoid/Anti-pattern**: Using context.Background() everywhere
```go
// ❌ Bad: Ignoring passed context
func (s *Service) DoWork(ctx context.Context) error {
    // Using Background() ignores cancellation!
    result, err := s.client.Call(context.Background(), request)
}

// ✅ Good: Propagate context
func (s *Service) DoWork(ctx context.Context) error {
    result, err := s.client.Call(ctx, request)
}
```

---

## G.3 Goroutines and Channels
- **Use when**: Concurrent operations, fan-out/fan-in patterns
- **Context**: Use channels for communication, sync primitives for shared state
```go
// Worker pool pattern
func ProcessItems(ctx context.Context, items []Item, workers int) ([]Result, error) {
    jobs := make(chan Item, len(items))
    results := make(chan Result, len(items))
    errs := make(chan error, 1)
    
    // Start workers
    var wg sync.WaitGroup
    for i := 0; i < workers; i++ {
        wg.Add(1)
        go func() {
            defer wg.Done()
            for item := range jobs {
                select {
                case <-ctx.Done():
                    return
                default:
                    result, err := processItem(ctx, item)
                    if err != nil {
                        select {
                        case errs <- err:
                        default:
                        }
                        return
                    }
                    results <- result
                }
            }
        }()
    }
    
    // Send jobs
    for _, item := range items {
        jobs <- item
    }
    close(jobs)
    
    // Wait for completion
    wg.Wait()
    close(results)
    close(errs)
    
    // Check for errors
    if err := <-errs; err != nil {
        return nil, err
    }
    
    // Collect results
    var output []Result
    for result := range results {
        output = append(output, result)
    }
    return output, nil
}

// Fan-out/Fan-in pattern
func FanOut(ctx context.Context, input <-chan int, workers int) <-chan int {
    outputs := make([]<-chan int, workers)
    
    for i := 0; i < workers; i++ {
        outputs[i] = worker(ctx, input)
    }
    
    return fanIn(ctx, outputs...)
}

func worker(ctx context.Context, input <-chan int) <-chan int {
    output := make(chan int)
    go func() {
        defer close(output)
        for n := range input {
            select {
            case <-ctx.Done():
                return
            case output <- n * 2:
            }
        }
    }()
    return output
}

func fanIn(ctx context.Context, channels ...<-chan int) <-chan int {
    var wg sync.WaitGroup
    output := make(chan int)
    
    multiplex := func(c <-chan int) {
        defer wg.Done()
        for n := range c {
            select {
            case <-ctx.Done():
                return
            case output <- n:
            }
        }
    }
    
    wg.Add(len(channels))
    for _, c := range channels {
        go multiplex(c)
    }
    
    go func() {
        wg.Wait()
        close(output)
    }()
    
    return output
}

// errgroup for coordinated goroutines
import "golang.org/x/sync/errgroup"

func FetchAllData(ctx context.Context, userID string) (*UserData, error) {
    g, ctx := errgroup.WithContext(ctx)
    
    var user *User
    var orders []Order
    var reviews []Review
    
    g.Go(func() error {
        var err error
        user, err = fetchUser(ctx, userID)
        return err
    })
    
    g.Go(func() error {
        var err error
        orders, err = fetchOrders(ctx, userID)
        return err
    })
    
    g.Go(func() error {
        var err error
        reviews, err = fetchReviews(ctx, userID)
        return err
    })
    
    if err := g.Wait(); err != nil {
        return nil, err
    }
    
    return &UserData{User: user, Orders: orders, Reviews: reviews}, nil
}
```
- **Avoid/Anti-pattern**: Goroutine leaks
```go
// ❌ Bad: Goroutine leak - channel never read
func bad() {
    ch := make(chan int)
    go func() {
        ch <- 1 // Blocked forever if no receiver
    }()
    // Forgot to receive from ch
}

// ✅ Good: Use buffered channel or ensure receiver
func good() {
    ch := make(chan int, 1) // Buffered
    go func() {
        ch <- 1 // Won't block
    }()
}
```

---

## G.4 Interface Design
- **Use when**: Defining contracts between components
- **Context**: Accept interfaces, return concrete types; small interfaces
```go
// Small, focused interfaces
type Reader interface {
    Read(p []byte) (n int, err error)
}

type Writer interface {
    Write(p []byte) (n int, err error)
}

type ReadWriter interface {
    Reader
    Writer
}

// Repository interface - accept interface
type UserRepository interface {
    FindByID(ctx context.Context, id string) (*User, error)
    FindByEmail(ctx context.Context, email string) (*User, error)
    Save(ctx context.Context, user *User) error
    Delete(ctx context.Context, id string) error
}

// Service accepts interface, returns concrete
type UserService struct {
    repo   UserRepository  // Interface
    events EventPublisher  // Interface
}

func NewUserService(repo UserRepository, events EventPublisher) *UserService {
    return &UserService{repo: repo, events: events}
}

// Concrete implementation
type PostgresUserRepository struct {
    db *sql.DB
}

func NewPostgresUserRepository(db *sql.DB) *PostgresUserRepository {
    return &PostgresUserRepository{db: db}
}

func (r *PostgresUserRepository) FindByID(ctx context.Context, id string) (*User, error) {
    // Implementation
}

// Interface for testing
type mockUserRepository struct {
    users map[string]*User
}

func (m *mockUserRepository) FindByID(ctx context.Context, id string) (*User, error) {
    if user, ok := m.users[id]; ok {
        return user, nil
    }
    return nil, ErrNotFound
}

// Functional options with interface
type Option func(*Config)

func WithTimeout(d time.Duration) Option {
    return func(c *Config) {
        c.timeout = d
    }
}

func WithRetries(n int) Option {
    return func(c *Config) {
        c.retries = n
    }
}

func NewClient(opts ...Option) *Client {
    cfg := defaultConfig()
    for _, opt := range opts {
        opt(cfg)
    }
    return &Client{config: cfg}
}
```
- **Avoid/Anti-pattern**: Large interfaces, exporting interfaces unnecessarily
```go
// ❌ Bad: Too large, hard to implement/mock
type UserManager interface {
    FindByID(ctx context.Context, id string) (*User, error)
    FindByEmail(ctx context.Context, email string) (*User, error)
    FindAll(ctx context.Context) ([]*User, error)
    Save(ctx context.Context, user *User) error
    Update(ctx context.Context, user *User) error
    Delete(ctx context.Context, id string) error
    SendEmail(ctx context.Context, id string, subject string) error
    ValidatePassword(password string) error
    // ... 20 more methods
}
```

---

## G.5 Table-Driven Tests
- **Use when**: Testing functions with multiple input/output scenarios
- **Context**: Use test tables with descriptive names
```go
func TestCalculateDiscount(t *testing.T) {
    tests := []struct {
        name     string
        amount   float64
        userType string
        want     float64
        wantErr  error
    }{
        {
            name:     "regular user no discount",
            amount:   100.0,
            userType: "regular",
            want:     100.0,
            wantErr:  nil,
        },
        {
            name:     "premium user 10% discount",
            amount:   100.0,
            userType: "premium",
            want:     90.0,
            wantErr:  nil,
        },
        {
            name:     "vip user 20% discount",
            amount:   100.0,
            userType: "vip",
            want:     80.0,
            wantErr:  nil,
        },
        {
            name:     "negative amount returns error",
            amount:   -50.0,
            userType: "regular",
            want:     0,
            wantErr:  ErrInvalidAmount,
        },
        {
            name:     "unknown user type returns error",
            amount:   100.0,
            userType: "unknown",
            want:     0,
            wantErr:  ErrUnknownUserType,
        },
    }
    
    for _, tt := range tests {
        t.Run(tt.name, func(t *testing.T) {
            got, err := CalculateDiscount(tt.amount, tt.userType)
            
            if !errors.Is(err, tt.wantErr) {
                t.Errorf("CalculateDiscount() error = %v, wantErr %v", err, tt.wantErr)
                return
            }
            
            if got != tt.want {
                t.Errorf("CalculateDiscount() = %v, want %v", got, tt.want)
            }
        })
    }
}

// With testify for cleaner assertions
func TestUserService_CreateUser(t *testing.T) {
    tests := []struct {
        name      string
        setupMock func(*mockUserRepository)
        input     CreateUserRequest
        want      *User
        wantErr   string
    }{
        {
            name: "success",
            setupMock: func(m *mockUserRepository) {
                m.On("ExistsByEmail", mock.Anything, "test@example.com").Return(false, nil)
                m.On("Save", mock.Anything, mock.AnythingOfType("*User")).Return(nil)
            },
            input:   CreateUserRequest{Email: "test@example.com", Name: "Test"},
            want:    &User{Email: "test@example.com", Name: "Test"},
            wantErr: "",
        },
        {
            name: "duplicate email",
            setupMock: func(m *mockUserRepository) {
                m.On("ExistsByEmail", mock.Anything, "existing@example.com").Return(true, nil)
            },
            input:   CreateUserRequest{Email: "existing@example.com", Name: "Test"},
            want:    nil,
            wantErr: "email already exists",
        },
    }
    
    for _, tt := range tests {
        t.Run(tt.name, func(t *testing.T) {
            mockRepo := new(mockUserRepository)
            tt.setupMock(mockRepo)
            
            service := NewUserService(mockRepo, new(mockEventPublisher))
            got, err := service.CreateUser(context.Background(), tt.input)
            
            if tt.wantErr != "" {
                require.Error(t, err)
                assert.Contains(t, err.Error(), tt.wantErr)
            } else {
                require.NoError(t, err)
                assert.Equal(t, tt.want.Email, got.Email)
            }
            
            mockRepo.AssertExpectations(t)
        })
    }
}
```

---

## G.6 HTTP Handler Patterns
- **Use when**: Building HTTP APIs
- **Context**: Use http.HandlerFunc, middleware chains, structured responses
```go
// Handler with proper error handling
type Handler struct {
    userService *UserService
    logger      *slog.Logger
}

func (h *Handler) GetUser(w http.ResponseWriter, r *http.Request) {
    ctx := r.Context()
    userID := r.PathValue("id") // Go 1.22+
    
    user, err := h.userService.GetUser(ctx, userID)
    if err != nil {
        h.handleError(w, r, err)
        return
    }
    
    h.respondJSON(w, http.StatusOK, user)
}

func (h *Handler) CreateUser(w http.ResponseWriter, r *http.Request) {
    ctx := r.Context()
    
    var req CreateUserRequest
    if err := json.NewDecoder(r.Body).Decode(&req); err != nil {
        h.respondError(w, http.StatusBadRequest, "invalid request body")
        return
    }
    
    if err := req.Validate(); err != nil {
        h.respondError(w, http.StatusBadRequest, err.Error())
        return
    }
    
    user, err := h.userService.CreateUser(ctx, req)
    if err != nil {
        h.handleError(w, r, err)
        return
    }
    
    h.respondJSON(w, http.StatusCreated, user)
}

func (h *Handler) respondJSON(w http.ResponseWriter, status int, data any) {
    w.Header().Set("Content-Type", "application/json")
    w.WriteHeader(status)
    if err := json.NewEncoder(w).Encode(data); err != nil {
        h.logger.Error("failed to encode response", "error", err)
    }
}

func (h *Handler) respondError(w http.ResponseWriter, status int, message string) {
    h.respondJSON(w, status, map[string]string{"error": message})
}

func (h *Handler) handleError(w http.ResponseWriter, r *http.Request, err error) {
    switch {
    case errors.Is(err, ErrNotFound):
        h.respondError(w, http.StatusNotFound, "resource not found")
    case errors.Is(err, ErrUnauthorized):
        h.respondError(w, http.StatusUnauthorized, "unauthorized")
    case errors.Is(err, context.DeadlineExceeded):
        h.respondError(w, http.StatusGatewayTimeout, "request timeout")
    default:
        h.logger.Error("internal error", 
            "error", err, 
            "path", r.URL.Path,
            "method", r.Method,
            "requestID", RequestIDFrom(r.Context()))
        h.respondError(w, http.StatusInternalServerError, "internal server error")
    }
}

// Middleware pattern
func LoggingMiddleware(logger *slog.Logger) func(http.Handler) http.Handler {
    return func(next http.Handler) http.Handler {
        return http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
            start := time.Now()
            
            // Wrap response writer to capture status
            wrapped := &responseWriter{ResponseWriter: w, status: http.StatusOK}
            
            next.ServeHTTP(wrapped, r)
            
            logger.Info("request completed",
                "method", r.Method,
                "path", r.URL.Path,
                "status", wrapped.status,
                "duration", time.Since(start),
                "requestID", RequestIDFrom(r.Context()))
        })
    }
}

type responseWriter struct {
    http.ResponseWriter
    status int
}

func (rw *responseWriter) WriteHeader(status int) {
    rw.status = status
    rw.ResponseWriter.WriteHeader(status)
}

// Router setup
func SetupRoutes(h *Handler) http.Handler {
    mux := http.NewServeMux()
    
    mux.HandleFunc("GET /api/users/{id}", h.GetUser)
    mux.HandleFunc("POST /api/users", h.CreateUser)
    mux.HandleFunc("PUT /api/users/{id}", h.UpdateUser)
    mux.HandleFunc("DELETE /api/users/{id}", h.DeleteUser)
    
    // Apply middleware
    var handler http.Handler = mux
    handler = LoggingMiddleware(h.logger)(handler)
    handler = RequestIDMiddleware(handler)
    handler = RecoveryMiddleware(h.logger)(handler)
    
    return handler
}
```

---

========================================
SECTION 4: DEVOPS PATTERNS
========================================

## D.1 Terraform Module Design
- **Use when**: Creating reusable infrastructure components
- **Context**: Use modules with clear inputs/outputs, version constraints
```hcl
# modules/aws-vpc/main.tf
terraform {
  required_version = ">= 1.5"
  required_providers {
    aws = {
      source  = "hashicorp/aws"
      version = "~> 5.0"
    }
  }
}

variable "name" {
  type        = string
  description = "Name prefix for all resources"
}

variable "cidr_block" {
  type        = string
  description = "CIDR block for VPC"
  default     = "10.0.0.0/16"
  
  validation {
    condition     = can(cidrhost(var.cidr_block, 0))
    error_message = "Must be a valid CIDR block."
  }
}

variable "availability_zones" {
  type        = list(string)
  description = "List of availability zones"
  
  validation {
    condition     = length(var.availability_zones) >= 2
    error_message = "At least 2 availability zones required for HA."
  }
}

variable "tags" {
  type        = map(string)
  description = "Tags to apply to all resources"
  default     = {}
}

locals {
  common_tags = merge(var.tags, {
    ManagedBy = "terraform"
    Module    = "aws-vpc"
  })
}

resource "aws_vpc" "main" {
  cidr_block           = var.cidr_block
  enable_dns_hostnames = true
  enable_dns_support   = true
  
  tags = merge(local.common_tags, {
    Name = "${var.name}-vpc"
  })
}

resource "aws_subnet" "public" {
  count             = length(var.availability_zones)
  vpc_id            = aws_vpc.main.id
  cidr_block        = cidrsubnet(var.cidr_block, 8, count.index)
  availability_zone = var.availability_zones[count.index]
  
  map_public_ip_on_launch = true
  
  tags = merge(local.common_tags, {
    Name = "${var.name}-public-${var.availability_zones[count.index]}"
    Type = "public"
  })
}

resource "aws_subnet" "private" {
  count             = length(var.availability_zones)
  vpc_id            = aws_vpc.main.id
  cidr_block        = cidrsubnet(var.cidr_block, 8, count.index + length(var.availability_zones))
  availability_zone = var.availability_zones[count.index]
  
  tags = merge(local.common_tags, {
    Name = "${var.name}-private-${var.availability_zones[count.index]}"
    Type = "private"
  })
}

output "vpc_id" {
  description = "The ID of the VPC"
  value       = aws_vpc.main.id
}

output "public_subnet_ids" {
  description = "List of public subnet IDs"
  value       = aws_subnet.public[*].id
}

output "private_subnet_ids" {
  description = "List of private subnet IDs"
  value       = aws_subnet.private[*].id
}

# Usage in root module
module "vpc" {
  source = "./modules/aws-vpc"
  
  name               = "production"
  cidr_block         = "10.0.0.0/16"
  availability_zones = ["us-east-1a", "us-east-1b", "us-east-1c"]
  
  tags = {
    Environment = "production"
    Project     = "myapp"
  }
}
```
- **Avoid/Anti-pattern**: Hardcoded values, no validation
```hcl
# ❌ Bad: Hardcoded values, no modularity
resource "aws_vpc" "main" {
  cidr_block = "10.0.0.0/16"  # Hardcoded
  tags = {
    Name = "my-vpc"  # Hardcoded
  }
}
```

---

## D.2 Docker Multi-Stage Builds
- **Use when**: Building production container images
- **Context**: Separate build and runtime stages, minimize image size
```dockerfile
# syntax=docker/dockerfile:1.4

# Stage 1: Build
FROM golang:1.22-alpine AS builder

WORKDIR /app

# Cache dependencies
COPY go.mod go.sum ./
RUN go mod download

# Copy source and build
COPY . .
RUN CGO_ENABLED=0 GOOS=linux go build -ldflags="-s -w" -o /app/server ./cmd/server

# Stage 2: Runtime
FROM gcr.io/distroless/static:nonroot

# Copy binary from builder
COPY --from=builder /app/server /server

# Copy config files if needed
COPY --from=builder /app/config /config

# Use non-root user (distroless default is nonroot)
USER nonroot:nonroot

# Expose port
EXPOSE 8080

# Health check
HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
  CMD ["/server", "healthcheck"]

# Run
ENTRYPOINT ["/server"]
```

```dockerfile
# Node.js example
FROM node:20-alpine AS deps
WORKDIR /app
COPY package*.json ./
RUN npm ci --only=production

FROM node:20-alpine AS builder
WORKDIR /app
COPY package*.json ./
RUN npm ci
COPY . .
RUN npm run build

FROM node:20-alpine AS runner
WORKDIR /app

ENV NODE_ENV=production
ENV PORT=3000

RUN addgroup --system --gid 1001 nodejs && \
    adduser --system --uid 1001 nextjs

COPY --from=deps /app/node_modules ./node_modules
COPY --from=builder --chown=nextjs:nodejs /app/.next ./.next
COPY --from=builder /app/public ./public
COPY --from=builder /app/package.json ./package.json

USER nextjs

EXPOSE 3000

CMD ["node", "server.js"]
```
- **Avoid/Anti-pattern**: Single stage with dev dependencies
```dockerfile
# ❌ Bad: Includes dev dependencies, large image
FROM node:20
WORKDIR /app
COPY . .
RUN npm install  # Includes devDependencies
CMD ["npm", "start"]
```

---

## D.3 Kubernetes Deployment Best Practices
- **Use when**: Deploying applications to Kubernetes
- **Context**: Use resource limits, health checks, pod disruption budgets
```yaml
# deployment.yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: user-service
  labels:
    app: user-service
    version: v1
spec:
  replicas: 3
  strategy:
    type: RollingUpdate
    rollingUpdate:
      maxSurge: 1
      maxUnavailable: 0
  selector:
    matchLabels:
      app: user-service
  template:
    metadata:
      labels:
        app: user-service
        version: v1
      annotations:
        prometheus.io/scrape: "true"
        prometheus.io/port: "8080"
        prometheus.io/path: "/metrics"
    spec:
      serviceAccountName: user-service
      securityContext:
        runAsNonRoot: true
        runAsUser: 1000
        fsGroup: 1000
      containers:
        - name: user-service
          image: myregistry/user-service:v1.2.3
          imagePullPolicy: IfNotPresent
          ports:
            - name: http
              containerPort: 8080
              protocol: TCP
          env:
            - name: DATABASE_URL
              valueFrom:
                secretKeyRef:
                  name: user-service-secrets
                  key: database-url
            - name: POD_NAME
              valueFrom:
                fieldRef:
                  fieldPath: metadata.name
          resources:
            requests:
              cpu: 100m
              memory: 128Mi
            limits:
              cpu: 500m
              memory: 512Mi
          livenessProbe:
            httpGet:
              path: /healthz
              port: http
            initialDelaySeconds: 15
            periodSeconds: 20
            timeoutSeconds: 5
            failureThreshold: 3
          readinessProbe:
            httpGet:
              path: /ready
              port: http
            initialDelaySeconds: 5
            periodSeconds: 10
            timeoutSeconds: 3
            failureThreshold: 3
          securityContext:
            allowPrivilegeEscalation: false
            readOnlyRootFilesystem: true
            capabilities:
              drop:
                - ALL
          volumeMounts:
            - name: tmp
              mountPath: /tmp
      volumes:
        - name: tmp
          emptyDir: {}
      affinity:
        podAntiAffinity:
          preferredDuringSchedulingIgnoredDuringExecution:
            - weight: 100
              podAffinityTerm:
                labelSelector:
                  matchLabels:
                    app: user-service
                topologyKey: kubernetes.io/hostname
      topologySpreadConstraints:
        - maxSkew: 1
          topologyKey: topology.kubernetes.io/zone
          whenUnsatisfiable: ScheduleAnyway
          labelSelector:
            matchLabels:
              app: user-service
---
apiVersion: policy/v1
kind: PodDisruptionBudget
metadata:
  name: user-service-pdb
spec:
  minAvailable: 2
  selector:
    matchLabels:
      app: user-service
---
apiVersion: autoscaling/v2
kind: HorizontalPodAutoscaler
metadata:
  name: user-service-hpa
spec:
  scaleTargetRef:
    apiVersion: apps/v1
    kind: Deployment
    name: user-service
  minReplicas: 3
  maxReplicas: 10
  metrics:
    - type: Resource
      resource:
        name: cpu
        target:
          type: Utilization
          averageUtilization: 70
    - type: Resource
      resource:
        name: memory
        target:
          type: Utilization
          averageUtilization: 80
```

---

## D.4 GitHub Actions CI/CD
- **Use when**: Automating build, test, and deployment pipelines
- **Context**: Use reusable workflows, matrix builds, caching
```yaml
# .github/workflows/ci.yml
name: CI

on:
  push:
    branches: [main, develop]
  pull_request:
    branches: [main]

concurrency:
  group: ${{ github.workflow }}-${{ github.ref }}
  cancel-in-progress: true

env:
  GO_VERSION: '1.22'
  NODE_VERSION: '20'

jobs:
  lint:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      
      - name: Set up Go
        uses: actions/setup-go@v5
        with:
          go-version: ${{ env.GO_VERSION }}
          cache: true
      
      - name: Run golangci-lint
        uses: golangci/golangci-lint-action@v4
        with:
          version: latest
          args: --timeout=5m

  test:
    runs-on: ubuntu-latest
    needs: lint
    services:
      postgres:
        image: postgres:15
        env:
          POSTGRES_USER: test
          POSTGRES_PASSWORD: test
          POSTGRES_DB: test
        ports:
          - 5432:5432
        options: >-
          --health-cmd pg_isready
          --health-interval 10s
          --health-timeout 5s
          --health-retries 5
    
    steps:
      - uses: actions/checkout@v4
      
      - name: Set up Go
        uses: actions/setup-go@v5
        with:
          go-version: ${{ env.GO_VERSION }}
          cache: true
      
      - name: Run tests
        env:
          DATABASE_URL: postgres://test:test@localhost:5432/test?sslmode=disable
        run: |
          go test -v -race -coverprofile=coverage.out ./...
          go tool cover -func=coverage.out
      
      - name: Upload coverage
        uses: codecov/codecov-action@v4
        with:
          files: coverage.out
          fail_ci_if_error: true

  build:
    runs-on: ubuntu-latest
    needs: test
    permissions:
      contents: read
      packages: write
    
    steps:
      - uses: actions/checkout@v4
      
      - name: Set up Docker Buildx
        uses: docker/setup-buildx-action@v3
      
      - name: Login to GitHub Container Registry
        uses: docker/login-action@v3
        with:
          registry: ghcr.io
          username: ${{ github.actor }}
          password: ${{ secrets.GITHUB_TOKEN }}
      
      - name: Extract metadata
        id: meta
        uses: docker/metadata-action@v5
        with:
          images: ghcr.io/${{ github.repository }}
          tags: |
            type=ref,event=branch
            type=ref,event=pr
            type=sha,prefix=
            type=semver,pattern={{version}}
      
      - name: Build and push
        uses: docker/build-push-action@v5
        with:
          context: .
          push: ${{ github.event_name != 'pull_request' }}
          tags: ${{ steps.meta.outputs.tags }}
          labels: ${{ steps.meta.outputs.labels }}
          cache-from: type=gha
          cache-to: type=gha,mode=max

  deploy:
    runs-on: ubuntu-latest
    needs: build
    if: github.ref == 'refs/heads/main'
    environment: production
    
    steps:
      - uses: actions/checkout@v4
      
      - name: Deploy to Kubernetes
        uses: azure/k8s-deploy@v4
        with:
          manifests: |
            k8s/deployment.yaml
            k8s/service.yaml
          images: |
            ghcr.io/${{ github.repository }}:${{ github.sha }}
```

---

========================================
SECTION 5: APACHE KAFKA PATTERNS
========================================

## K.1 Idempotent Producer
- **Use when**: Ensuring exactly-once delivery semantics
- **Context**: Enable idempotence, configure retries and acks
```java
// Idempotent producer configuration
public class KafkaProducerConfig {
    
    @Bean
    public ProducerFactory<String, Object> producerFactory() {
        Map<String, Object> config = new HashMap<>();
        
        config.put(ProducerConfig.BOOTSTRAP_SERVERS_CONFIG, bootstrapServers);
        config.put(ProducerConfig.KEY_SERIALIZER_CLASS_CONFIG, StringSerializer.class);
        config.put(ProducerConfig.VALUE_SERIALIZER_CLASS_CONFIG, JsonSerializer.class);
        
        // Idempotence settings
        config.put(ProducerConfig.ENABLE_IDEMPOTENCE_CONFIG, true);
        config.put(ProducerConfig.ACKS_CONFIG, "all");
        config.put(ProducerConfig.RETRIES_CONFIG, Integer.MAX_VALUE);
        config.put(ProducerConfig.MAX_IN_FLIGHT_REQUESTS_PER_CONNECTION, 5);
        
        // Delivery timeout
        config.put(ProducerConfig.DELIVERY_TIMEOUT_MS_CONFIG, 120000);
        config.put(ProducerConfig.REQUEST_TIMEOUT_MS_CONFIG, 30000);
        
        // Batching for throughput
        config.put(ProducerConfig.BATCH_SIZE_CONFIG, 16384);
        config.put(ProducerConfig.LINGER_MS_CONFIG, 10);
        config.put(ProducerConfig.COMPRESSION_TYPE_CONFIG, "snappy");
        
        return new DefaultKafkaProducerFactory<>(config);
    }
    
    @Bean
    public KafkaTemplate<String, Object> kafkaTemplate() {
        return new KafkaTemplate<>(producerFactory());
    }
}

// Producer service with proper error handling
@Service
@Slf4j
public class OrderEventProducer {
    private final KafkaTemplate<String, Object> kafkaTemplate;
    
    public CompletableFuture<SendResult<String, Object>> publishOrderCreated(OrderCreatedEvent event) {
        return kafkaTemplate.send("orders.created.v1", event.orderId(), event)
            .whenComplete((result, ex) -> {
                if (ex != null) {
                    log.error("Failed to publish OrderCreated event", ex);
                } else {
                    log.info("OrderCreated event published: topic={}, partition={}, offset={}",
                        result.getRecordMetadata().topic(),
                        result.getRecordMetadata().partition(),
                        result.getRecordMetadata().offset());
                }
            });
    }
}
```

---

## K.2 Consumer Group with Manual Commit
- **Use when**: Controlling offset commits for at-least-once processing
- **Context**: Disable auto-commit, commit after processing
```java
@Configuration
public class KafkaConsumerConfig {
    
    @Bean
    public ConsumerFactory<String, OrderCreatedEvent> consumerFactory() {
        Map<String, Object> config = new HashMap<>();
        
        config.put(ConsumerConfig.BOOTSTRAP_SERVERS_CONFIG, bootstrapServers);
        config.put(ConsumerConfig.GROUP_ID_CONFIG, "order-processor-group");
        config.put(ConsumerConfig.KEY_DESERIALIZER_CLASS_CONFIG, StringDeserializer.class);
        config.put(ConsumerConfig.VALUE_DESERIALIZER_CLASS_CONFIG, JsonDeserializer.class);
        
        // Manual commit
        config.put(ConsumerConfig.ENABLE_AUTO_COMMIT_CONFIG, false);
        
        // Consumer tuning
        config.put(ConsumerConfig.MAX_POLL_RECORDS_CONFIG, 100);
        config.put(ConsumerConfig.MAX_POLL_INTERVAL_MS_CONFIG, 300000);
        config.put(ConsumerConfig.SESSION_TIMEOUT_MS_CONFIG, 45000);
        config.put(ConsumerConfig.HEARTBEAT_INTERVAL_MS_CONFIG, 15000);
        
        // Start from earliest if no committed offset
        config.put(ConsumerConfig.AUTO_OFFSET_RESET_CONFIG, "earliest");
        
        // Deserializer config
        config.put(JsonDeserializer.TRUSTED_PACKAGES, "com.example.events");
        
        return new DefaultKafkaConsumerFactory<>(config);
    }
    
    @Bean
    public ConcurrentKafkaListenerContainerFactory<String, OrderCreatedEvent> 
            kafkaListenerContainerFactory() {
        
        ConcurrentKafkaListenerContainerFactory<String, OrderCreatedEvent> factory =
            new ConcurrentKafkaListenerContainerFactory<>();
        
        factory.setConsumerFactory(consumerFactory());
        factory.setConcurrency(3);
        factory.getContainerProperties().setAckMode(AckMode.MANUAL_IMMEDIATE);
        
        // Error handling
        factory.setCommonErrorHandler(new DefaultErrorHandler(
            new DeadLetterPublishingRecoverer(kafkaTemplate),
            new FixedBackOff(1000L, 3L)
        ));
        
        return factory;
    }
}

@Service
@Slf4j
public class OrderEventConsumer {
    private final OrderService orderService;
    private final IdempotencyService idempotencyService;
    
    @KafkaListener(
        topics = "orders.created.v1",
        groupId = "order-processor-group",
        containerFactory = "kafkaListenerContainerFactory"
    )
    public void handleOrderCreated(
            @Payload OrderCreatedEvent event,
            @Header(KafkaHeaders.RECEIVED_KEY) String key,
            @Header(KafkaHeaders.RECEIVED_PARTITION) int partition,
            @Header(KafkaHeaders.OFFSET) long offset,
            Acknowledgment acknowledgment) {
        
        String idempotencyKey = String.format("%s-%d-%d", key, partition, offset);
        
        try {
            // Idempotency check
            if (idempotencyService.isDuplicate(idempotencyKey)) {
                log.info("Duplicate event detected, skipping: {}", idempotencyKey);
                acknowledgment.acknowledge();
                return;
            }
            
            // Process event
            orderService.processOrderCreated(event);
            
            // Mark as processed
            idempotencyService.markProcessed(idempotencyKey);
            
            // Commit offset
            acknowledgment.acknowledge();
            
            log.info("Successfully processed OrderCreated: orderId={}", event.orderId());
            
        } catch (RetryableException e) {
            log.warn("Retryable error processing event, will retry: {}", e.getMessage());
            throw e; // Let error handler retry
            
        } catch (Exception e) {
            log.error("Non-retryable error processing event: {}", e.getMessage(), e);
            acknowledgment.acknowledge(); // Acknowledge to skip
            // Send to DLQ is handled by error handler
        }
    }
}
```

---

## K.3 Kafka Streams with Windowing
- **Use when**: Processing streaming data with time-based aggregations
- **Context**: Use KStream/KTable, configure windowing
```java
@Configuration
@EnableKafkaStreams
public class KafkaStreamsConfig {
    
    @Bean(name = KafkaStreamsDefaultConfiguration.DEFAULT_STREAMS_CONFIG_BEAN_NAME)
    public KafkaStreamsConfiguration streamsConfig() {
        Map<String, Object> config = new HashMap<>();
        
        config.put(StreamsConfig.APPLICATION_ID_CONFIG, "order-analytics-app");
        config.put(StreamsConfig.BOOTSTRAP_SERVERS_CONFIG, bootstrapServers);
        config.put(StreamsConfig.DEFAULT_KEY_SERDE_CLASS_CONFIG, Serdes.String().getClass());
        config.put(StreamsConfig.DEFAULT_VALUE_SERDE_CLASS_CONFIG, JsonSerde.class);
        
        // Processing guarantee
        config.put(StreamsConfig.PROCESSING_GUARANTEE_CONFIG, StreamsConfig.EXACTLY_ONCE_V2);
        
        // State store
        config.put(StreamsConfig.STATE_DIR_CONFIG, "/tmp/kafka-streams");
        
        return new KafkaStreamsConfiguration(config);
    }
    
    @Bean
    public KStream<String, OrderEvent> orderAnalyticsStream(StreamsBuilder builder) {
        // Read from orders topic
        KStream<String, OrderEvent> orders = builder.stream(
            "orders.created.v1",
            Consumed.with(Serdes.String(), orderEventSerde())
        );
        
        // Count orders per customer in 1-hour tumbling windows
        orders
            .groupBy((key, order) -> order.customerId())
            .windowedBy(TimeWindows.ofSizeWithNoGrace(Duration.ofHours(1)))
            .count(Materialized.as("orders-per-customer-hourly"))
            .toStream()
            .map((windowedKey, count) -> KeyValue.pair(
                windowedKey.key(),
                new CustomerOrderCount(
                    windowedKey.key(),
                    windowedKey.window().start(),
                    windowedKey.window().end(),
                    count
                )
            ))
            .to("customer.order.counts.v1");
        
        // Calculate running total per product in 5-minute hopping windows
        orders
            .flatMapValues(order -> order.items())
            .groupBy((key, item) -> item.productId())
            .windowedBy(TimeWindows.ofSizeAndGrace(Duration.ofMinutes(5), Duration.ofMinutes(1))
                .advanceBy(Duration.ofMinutes(1)))
            .aggregate(
                () -> BigDecimal.ZERO,
                (key, item, total) -> total.add(item.total()),
                Materialized.<String, BigDecimal, WindowStore<Bytes, byte[]>>as("product-revenue")
                    .withKeySerde(Serdes.String())
                    .withValueSerde(bigDecimalSerde())
            )
            .toStream()
            .to("product.revenue.v1");
        
        return orders;
    }
}
```

---

## K.4 Schema Registry with Avro
- **Use when**: Schema evolution, type-safe serialization
- **Context**: Use Avro schemas with Confluent Schema Registry
```avro
// order_created.avsc
{
  "type": "record",
  "name": "OrderCreatedEvent",
  "namespace": "com.example.events",
  "fields": [
    {"name": "orderId", "type": "string"},
    {"name": "customerId", "type": "string"},
    {"name": "items", "type": {"type": "array", "items": {
      "type": "record",
      "name": "OrderItem",
      "fields": [
        {"name": "productId", "type": "string"},
        {"name": "quantity", "type": "int"},
        {"name": "price", "type": {"type": "bytes", "logicalType": "decimal", "precision": 10, "scale": 2}}
      ]
    }}},
    {"name": "total", "type": {"type": "bytes", "logicalType": "decimal", "precision": 12, "scale": 2}},
    {"name": "createdAt", "type": {"type": "long", "logicalType": "timestamp-millis"}},
    {"name": "version", "type": "int", "default": 1}
  ]
}
```

```java
// Producer configuration with Schema Registry
@Configuration
public class AvroKafkaConfig {
    
    @Value("${spring.kafka.properties.schema.registry.url}")
    private String schemaRegistryUrl;
    
    @Bean
    public ProducerFactory<String, OrderCreatedEvent> avroProducerFactory() {
        Map<String, Object> config = new HashMap<>();
        
        config.put(ProducerConfig.BOOTSTRAP_SERVERS_CONFIG, bootstrapServers);
        config.put(ProducerConfig.KEY_SERIALIZER_CLASS_CONFIG, StringSerializer.class);
        config.put(ProducerConfig.VALUE_SERIALIZER_CLASS_CONFIG, KafkaAvroSerializer.class);
        
        // Schema Registry
        config.put(KafkaAvroSerializerConfig.SCHEMA_REGISTRY_URL_CONFIG, schemaRegistryUrl);
        config.put(KafkaAvroSerializerConfig.AUTO_REGISTER_SCHEMAS, true);
        
        // Subject naming strategy
        config.put(KafkaAvroSerializerConfig.VALUE_SUBJECT_NAME_STRATEGY, 
            TopicRecordNameStrategy.class.getName());
        
        return new DefaultKafkaProducerFactory<>(config);
    }
    
    @Bean
    public ConsumerFactory<String, OrderCreatedEvent> avroConsumerFactory() {
        Map<String, Object> config = new HashMap<>();
        
        config.put(ConsumerConfig.BOOTSTRAP_SERVERS_CONFIG, bootstrapServers);
        config.put(ConsumerConfig.KEY_DESERIALIZER_CLASS_CONFIG, StringDeserializer.class);
        config.put(ConsumerConfig.VALUE_DESERIALIZER_CLASS_CONFIG, KafkaAvroDeserializer.class);
        
        // Schema Registry
        config.put(KafkaAvroDeserializerConfig.SCHEMA_REGISTRY_URL_CONFIG, schemaRegistryUrl);
        config.put(KafkaAvroDeserializerConfig.SPECIFIC_AVRO_READER_CONFIG, true);
        
        return new DefaultKafkaConsumerFactory<>(config);
    }
}
```

---

## K.5 Testing Kafka with EmbeddedKafka
- **Use when**: Integration testing Kafka producers/consumers
- **Context**: Use EmbeddedKafka or Testcontainers
```java
@SpringBootTest
@EmbeddedKafka(
    partitions = 3,
    topics = {"orders.created.v1", "orders.created.v1.DLT"},
    brokerProperties = {
        "listeners=PLAINTEXT://localhost:9092",
        "port=9092"
    }
)
class OrderEventIT {
    
    @Autowired
    private KafkaTemplate<String, OrderCreatedEvent> kafkaTemplate;
    
    @Autowired
    private EmbeddedKafkaBroker embeddedKafkaBroker;
    
    @Test
    void shouldProcessOrderCreatedEvent() throws Exception {
        // Given
        var event = new OrderCreatedEvent(
            "order-123",
            "customer-456",
            List.of(new OrderItem("product-789", 2, new BigDecimal("29.99"))),
            new BigDecimal("59.98"),
            Instant.now()
        );
        
        // When
        kafkaTemplate.send("orders.created.v1", event.orderId(), event).get();
        
        // Then
        await().atMost(Duration.ofSeconds(10))
            .untilAsserted(() -> {
                var processedOrder = orderRepository.findById("order-123");
                assertThat(processedOrder).isPresent();
                assertThat(processedOrder.get().getStatus()).isEqualTo(OrderStatus.PROCESSING);
            });
    }
    
    @Test
    void shouldSendToDeadLetterTopicOnError() throws Exception {
        // Given - malformed event that will fail processing
        var badEvent = new OrderCreatedEvent(
            null, // Will cause validation error
            "customer-456",
            List.of(),
            BigDecimal.ZERO,
            Instant.now()
        );
        
        // Create consumer for DLT
        Map<String, Object> consumerProps = KafkaTestUtils.consumerProps(
            "test-group", "false", embeddedKafkaBroker
        );
        var consumer = new DefaultKafkaConsumerFactory<String, OrderCreatedEvent>(consumerProps)
            .createConsumer();
        consumer.subscribe(List.of("orders.created.v1.DLT"));
        
        // When
        kafkaTemplate.send("orders.created.v1", "bad-order", badEvent).get();
        
        // Then
        var records = KafkaTestUtils.getRecords(consumer, Duration.ofSeconds(10));
        assertThat(records.count()).isEqualTo(1);
    }
}

// Alternative: Testcontainers
@SpringBootTest
@Testcontainers
class OrderEventTestcontainersIT {
    
    @Container
    static KafkaContainer kafka = new KafkaContainer(
        DockerImageName.parse("confluentinc/cp-kafka:7.5.0")
    );
    
    @Container
    static GenericContainer<?> schemaRegistry = new GenericContainer<>(
        DockerImageName.parse("confluentinc/cp-schema-registry:7.5.0")
    )
    .withExposedPorts(8081)
    .withEnv("SCHEMA_REGISTRY_HOST_NAME", "schema-registry")
    .withEnv("SCHEMA_REGISTRY_KAFKASTORE_BOOTSTRAP_SERVERS", 
        "PLAINTEXT://" + kafka.getNetworkAliases().get(0) + ":9092")
    .dependsOn(kafka)
    .withNetwork(kafka.getNetwork());
    
    @DynamicPropertySource
    static void kafkaProperties(DynamicPropertyRegistry registry) {
        registry.add("spring.kafka.bootstrap-servers", kafka::getBootstrapServers);
        registry.add("spring.kafka.properties.schema.registry.url", 
            () -> "http://localhost:" + schemaRegistry.getMappedPort(8081));
    }
}
```

---

## QUALITY CHECKLISTS

### ✅ TypeScript Code Quality
- [ ] strict mode enabled in tsconfig
- [ ] No `any` types (use `unknown` for uncertain types)
- [ ] Branded types for domain IDs
- [ ] Zod schemas for external data validation
- [ ] Discriminated unions for state machines
- [ ] Proper error handling (no unhandled rejections)

### ✅ Java/Spring Boot Quality
- [ ] Records for DTOs and value objects
- [ ] Optional for nullable returns only
- [ ] Constructor injection (no field injection)
- [ ] JPA entities with proper equals/hashCode
- [ ] Sealed classes for closed hierarchies
- [ ] Virtual threads for I/O-bound operations (Java 21+)

### ✅ Go Code Quality
- [ ] Errors wrapped with context
- [ ] Context propagated through call chain
- [ ] Interfaces accepted, structs returned
- [ ] Table-driven tests
- [ ] Goroutines properly managed (no leaks)
- [ ] golangci-lint clean

### ✅ Kafka Quality
- [ ] Idempotent producers enabled
- [ ] Manual offset commit after processing
- [ ] Dead letter topics configured
- [ ] Schema Registry for schema evolution
- [ ] Consumer group tuning (poll interval, session timeout)
- [ ] Monitoring (consumer lag, partition distribution)

### ✅ DevOps Quality
- [ ] Infrastructure as Code (Terraform/Pulumi)
- [ ] Multi-stage Docker builds
- [ ] Kubernetes resource limits and health checks
- [ ] CI/CD with proper caching
- [ ] Secrets management (not in code)
- [ ] Observability (metrics, logs, traces)

---

## Tool Recommendations (2024-2025)

| Category | Recommended | Alternative |
|----------|-------------|-------------|
| **TS Testing** | Vitest | Jest |
| **TS Validation** | Zod | Yup |
| **TS State** | Zustand | Redux Toolkit |
| **Java Build** | Gradle | Maven |
| **Java Testing** | JUnit 5 + AssertJ | - |
| **Go Testing** | testify | - |
| **Go Linting** | golangci-lint | - |
| **IaC** | Terraform | Pulumi |
| **K8s** | Helm + Kustomize | - |
| **CI/CD** | GitHub Actions | GitLab CI |
| **Kafka Testing** | Testcontainers | EmbeddedKafka |
| **Schema** | Avro + Schema Registry | Protobuf |


# S77-EXTENDED: Comprehensive Idiomatic Patterns
## TypeScript • JavaScript • Java • Spring Boot • Golang • DevOps • Apache Kafka

**Total Patterns: 600+**
**Structure: Use when → Context → Avoid/Anti-pattern**
**TDD Workflow: STUB → RED → GREEN → REFACTOR**

========================================
SECTION TS: TYPESCRIPT PATTERNS (150+)
========================================

## TS.0 WORKSPACE AND PROJECT SETUP

TS.0.1 Strict TypeScript Configuration
- Use when: Starting any new TypeScript project
- Context: Enable all strict checks for maximum type safety
```json
// tsconfig.json
{
  "compilerOptions": {
    "strict": true,
    "noUncheckedIndexedAccess": true,
    "noImplicitReturns": true,
    "noFallthroughCasesInSwitch": true,
    "noImplicitOverride": true,
    "exactOptionalPropertyTypes": true,
    "noPropertyAccessFromIndexSignature": true,
    "forceConsistentCasingInFileNames": true,
    "moduleResolution": "bundler",
    "module": "ESNext",
    "target": "ES2022",
    "lib": ["ES2022", "DOM", "DOM.Iterable"],
    "skipLibCheck": true,
    "declaration": true,
    "declarationMap": true,
    "sourceMap": true
  }
}
```
- Avoid/Anti-pattern: `"strict": false` or disabling checks piecemeal
```json
// ❌ Bad: Loose configuration
{
  "compilerOptions": {
    "strict": false,
    "noImplicitAny": false  // Defeats type safety purpose
  }
}
```

TS.0.2 Path Aliases for Clean Imports
- Use when: Project has deep nested directories
- Context: Configure path aliases to avoid `../../../` imports
```json
// tsconfig.json
{
  "compilerOptions": {
    "baseUrl": ".",
    "paths": {
      "@/*": ["src/*"],
      "@components/*": ["src/components/*"],
      "@utils/*": ["src/utils/*"],
      "@types/*": ["src/types/*"]
    }
  }
}
```
```typescript
// Usage
import { UserService } from '@/services/user.service';
import { Button } from '@components/ui/Button';
import { formatDate } from '@utils/date';
```
- Avoid/Anti-pattern: Relative path hell
```typescript
// ❌ Bad: Unmaintainable imports
import { UserService } from '../../../services/user.service';
import { Button } from '../../../../components/ui/Button';
```

TS.0.3 ESLint + Prettier Integration
- Use when: Enforcing code quality and consistency
- Context: Configure ESLint for logic, Prettier for formatting
```javascript
// eslint.config.js (flat config)
import eslint from '@eslint/js';
import tseslint from 'typescript-eslint';
import prettier from 'eslint-config-prettier';

export default tseslint.config(
  eslint.configs.recommended,
  ...tseslint.configs.strictTypeChecked,
  ...tseslint.configs.stylisticTypeChecked,
  prettier,
  {
    languageOptions: {
      parserOptions: {
        project: true,
        tsconfigRootDir: import.meta.dirname,
      },
    },
    rules: {
      '@typescript-eslint/no-unused-vars': ['error', { argsIgnorePattern: '^_' }],
      '@typescript-eslint/explicit-function-return-type': 'error',
      '@typescript-eslint/no-explicit-any': 'error',
      '@typescript-eslint/prefer-nullish-coalescing': 'error',
      '@typescript-eslint/prefer-optional-chain': 'error',
    },
  }
);
```

TS.0.4 Monorepo with Turborepo/Nx
- Use when: Multiple related packages in one repository
- Context: Configure workspace for shared dependencies and build caching
```json
// turbo.json
{
  "$schema": "https://turbo.build/schema.json",
  "globalDependencies": ["**/.env.*local"],
  "pipeline": {
    "build": {
      "dependsOn": ["^build"],
      "outputs": ["dist/**", ".next/**"]
    },
    "test": {
      "dependsOn": ["build"],
      "inputs": ["src/**", "test/**"]
    },
    "lint": {},
    "dev": {
      "cache": false,
      "persistent": true
    }
  }
}
```

TS.0.5 Vitest Configuration
- Use when: Setting up testing for TypeScript projects
- Context: Configure Vitest with TypeScript support
```typescript
// vitest.config.ts
import { defineConfig } from 'vitest/config';
import path from 'path';

export default defineConfig({
  test: {
    globals: true,
    environment: 'node',
    include: ['src/**/*.{test,spec}.{ts,tsx}'],
    coverage: {
      provider: 'v8',
      reporter: ['text', 'json', 'html'],
      exclude: ['node_modules/', 'dist/', '**/*.d.ts'],
      thresholds: {
        lines: 80,
        functions: 80,
        branches: 80,
        statements: 80,
      },
    },
    setupFiles: ['./src/test/setup.ts'],
  },
  resolve: {
    alias: {
      '@': path.resolve(__dirname, './src'),
    },
  },
});
```

---

## TS.1 CORE LANGUAGE IDIOMS

TS.1.1 Branded Types for Type-Safe Identifiers
- Use when: Preventing mixing of semantically different string/number IDs
- Context: Use intersection with unique symbol for nominal typing
```typescript
// Define branded types
declare const __brand: unique symbol;
type Brand<T, B> = T & { [__brand]: B };

type UserId = Brand<string, 'UserId'>;
type OrderId = Brand<string, 'OrderId'>;
type ProductId = Brand<string, 'ProductId'>;

// Constructor functions with validation
function UserId(id: string): UserId {
  if (!id.startsWith('usr_')) {
    throw new Error(`Invalid UserId format: ${id}`);
  }
  return id as UserId;
}

function OrderId(id: string): OrderId {
  if (!id.startsWith('ord_')) {
    throw new Error(`Invalid OrderId format: ${id}`);
  }
  return id as OrderId;
}

// Usage - compiler prevents mixing
function getUser(id: UserId): Promise<User> { /* ... */ }
function getOrder(id: OrderId): Promise<Order> { /* ... */ }

const userId = UserId('usr_123');
const orderId = OrderId('ord_456');

getUser(userId);   // ✅ OK
getUser(orderId);  // ❌ Compile error: OrderId not assignable to UserId
```
- Avoid/Anti-pattern: Plain strings for all IDs
```typescript
// ❌ Bad: No type safety
function getUser(id: string): Promise<User> { }
function getOrder(id: string): Promise<Order> { }

getUser(orderId); // No error, runtime bug
```
- Testing:
```typescript
describe('Branded Types', () => {
  it('should create valid UserId', () => {
    const id = UserId('usr_123');
    expect(id).toBe('usr_123');
  });

  it('should reject invalid UserId format', () => {
    expect(() => UserId('invalid')).toThrow('Invalid UserId format');
  });
});
```

TS.1.2 Discriminated Unions for State Machines
- Use when: Modeling mutually exclusive states with different data
- Context: Use literal type discriminant for exhaustive checking
```typescript
// Define state machine with discriminated union
type RequestState<T, E = Error> =
  | { status: 'idle' }
  | { status: 'loading'; startedAt: Date }
  | { status: 'success'; data: T; completedAt: Date }
  | { status: 'error'; error: E; failedAt: Date };

// Type guard for specific states
function isSuccess<T, E>(state: RequestState<T, E>): state is { status: 'success'; data: T; completedAt: Date } {
  return state.status === 'success';
}

// Exhaustive handler
function handleRequest<T>(state: RequestState<T>): string {
  switch (state.status) {
    case 'idle':
      return 'Ready to fetch';
    case 'loading':
      return `Loading since ${state.startedAt.toISOString()}`;
    case 'success':
      return `Got data: ${JSON.stringify(state.data)}`;
    case 'error':
      return `Failed: ${state.error.message}`;
    // No default needed - TypeScript ensures exhaustiveness
  }
}

// State transitions (type-safe)
function startLoading<T>(state: RequestState<T>): RequestState<T> {
  if (state.status === 'loading') return state; // Already loading
  return { status: 'loading', startedAt: new Date() };
}

function succeed<T>(state: RequestState<T>, data: T): RequestState<T> {
  if (state.status !== 'loading') {
    throw new Error('Can only succeed from loading state');
  }
  return { status: 'success', data, completedAt: new Date() };
}
```
- Avoid/Anti-pattern: Separate boolean flags
```typescript
// ❌ Bad: Impossible states representable
type BadState<T> = {
  isLoading: boolean;
  isError: boolean;
  isSuccess: boolean;
  data?: T;
  error?: Error;
};
// Can have isLoading: true AND isError: true - invalid!
// Can have isSuccess: true but data: undefined - invalid!
```

TS.1.3 Const Assertions for Literal Types
- Use when: Preserving literal types instead of widened types
- Context: Use `as const` for readonly literals
```typescript
// Without const assertion
const httpMethods = ['GET', 'POST', 'PUT', 'DELETE'];
// type: string[]

// With const assertion - preserves literals
const httpMethods = ['GET', 'POST', 'PUT', 'DELETE'] as const;
// type: readonly ['GET', 'POST', 'PUT', 'DELETE']

type HttpMethod = typeof httpMethods[number];
// type: 'GET' | 'POST' | 'PUT' | 'DELETE'

// Object const assertion
const config = {
  apiUrl: 'https://api.example.com',
  timeout: 5000,
  retries: 3,
} as const;
// All properties are readonly with literal types

// Useful for action types
const ActionTypes = {
  USER_LOGIN: 'USER_LOGIN',
  USER_LOGOUT: 'USER_LOGOUT',
  USER_UPDATE: 'USER_UPDATE',
} as const;

type ActionType = typeof ActionTypes[keyof typeof ActionTypes];
// type: 'USER_LOGIN' | 'USER_LOGOUT' | 'USER_UPDATE'

// Type-safe action creators
type Action = 
  | { type: typeof ActionTypes.USER_LOGIN; payload: { userId: string } }
  | { type: typeof ActionTypes.USER_LOGOUT }
  | { type: typeof ActionTypes.USER_UPDATE; payload: Partial<User> };
```

TS.1.4 Type Predicates for Custom Type Guards
- Use when: Runtime type checking with TypeScript type narrowing
- Context: Return `value is Type` for compiler to narrow types
```typescript
interface User {
  type: 'user';
  id: string;
  name: string;
  email: string;
}

interface Admin {
  type: 'admin';
  id: string;
  name: string;
  email: string;
  permissions: string[];
}

type Person = User | Admin;

// Type predicate
function isAdmin(person: Person): person is Admin {
  return person.type === 'admin';
}

function isUser(person: Person): person is User {
  return person.type === 'user';
}

// Usage with filter - correctly typed result
const people: Person[] = [/* ... */];
const admins: Admin[] = people.filter(isAdmin); // Correctly typed as Admin[]
const users: User[] = people.filter(isUser);    // Correctly typed as User[]

// Type guard for unknown data
function isPerson(value: unknown): value is Person {
  return (
    typeof value === 'object' &&
    value !== null &&
    'type' in value &&
    (value.type === 'user' || value.type === 'admin') &&
    'id' in value &&
    typeof value.id === 'string'
  );
}

// Usage in conditional
function getPermissions(person: Person): string[] {
  if (isAdmin(person)) {
    return person.permissions; // TypeScript knows this is Admin
  }
  return []; // TypeScript knows this is User
}
```
- Avoid/Anti-pattern: Boolean return without type predicate
```typescript
// ❌ Bad: No type narrowing
function isAdmin(person: Person): boolean {
  return person.type === 'admin';
}

const admins = people.filter(isAdmin); // Still typed as Person[], not Admin[]
```

TS.1.5 Generics with Constraints
- Use when: Writing reusable code that works with multiple types
- Context: Use `extends` to constrain type parameters
```typescript
// Basic constraint
function getProperty<T, K extends keyof T>(obj: T, key: K): T[K] {
  return obj[key];
}

const user = { name: 'John', age: 30, email: 'john@example.com' };
const name = getProperty(user, 'name'); // type: string
const age = getProperty(user, 'age');   // type: number
// getProperty(user, 'invalid');        // ❌ Compile error

// Multiple constraints
function merge<T extends object, U extends object>(a: T, b: U): T & U {
  return { ...a, ...b };
}

// Constraint with interface
interface HasId {
  id: string;
}

interface HasTimestamps {
  createdAt: Date;
  updatedAt: Date;
}

function updateEntity<T extends HasId & HasTimestamps>(
  entity: T,
  updates: Partial<Omit<T, 'id' | 'createdAt'>>
): T {
  return {
    ...entity,
    ...updates,
    updatedAt: new Date(),
  };
}

// Generic class with constraint
class Repository<T extends HasId> {
  private items = new Map<string, T>();

  save(item: T): void {
    this.items.set(item.id, item);
  }

  findById(id: string): T | undefined {
    return this.items.get(id);
  }

  findAll(): T[] {
    return Array.from(this.items.values());
  }
}

// Default type parameter
function createState<T = string>(initial: T): { value: T; set: (v: T) => void } {
  let value = initial;
  return {
    get value() { return value; },
    set(v: T) { value = v; },
  };
}
```

TS.1.6 Template Literal Types
- Use when: Creating string types from combinations
- Context: Use template literals for type-safe string patterns
```typescript
type HttpMethod = 'GET' | 'POST' | 'PUT' | 'DELETE' | 'PATCH';
type ApiVersion = 'v1' | 'v2' | 'v3';
type Resource = 'users' | 'orders' | 'products';

// Combine into endpoint pattern
type ApiEndpoint = `/${ApiVersion}/${Resource}`;
// '/v1/users' | '/v1/orders' | '/v1/products' | '/v2/users' | ...

// Event handler types
type EventName = 'click' | 'hover' | 'focus' | 'blur';
type EventHandler<T extends string> = `on${Capitalize<T>}`;

type ClickHandler = EventHandler<'click'>; // 'onClick'
type HoverHandler = EventHandler<'hover'>; // 'onHover'

// CSS utility types
type CSSProperty = 'margin' | 'padding' | 'border';
type CSSDirection = 'Top' | 'Right' | 'Bottom' | 'Left';
type CSSSpacing = `${CSSProperty}${CSSDirection}`;
// 'marginTop' | 'marginRight' | ... | 'borderLeft'

// Route parameter extraction
type ExtractParams<T extends string> = 
  T extends `${infer _Start}:${infer Param}/${infer Rest}`
    ? Param | ExtractParams<Rest>
    : T extends `${infer _Start}:${infer Param}`
      ? Param
      : never;

type UserRouteParams = ExtractParams<'/users/:userId/orders/:orderId'>;
// 'userId' | 'orderId'

// Type-safe route builder
type Route<T extends string> = {
  path: T;
  params: Record<ExtractParams<T>, string>;
};

function buildUrl<T extends string>(
  path: T,
  params: Record<ExtractParams<T>, string>
): string {
  let url: string = path;
  for (const [key, value] of Object.entries(params)) {
    url = url.replace(`:${key}`, value as string);
  }
  return url;
}

// Usage
const url = buildUrl('/users/:userId/orders/:orderId', {
  userId: 'u123',
  orderId: 'o456',
}); // '/users/u123/orders/o456'
```

TS.1.7 Mapped Types for Transformations
- Use when: Creating new types by transforming existing ones
- Context: Use mapped types with modifiers
```typescript
// Built-in utility types (for reference)
type Partial<T> = { [K in keyof T]?: T[K] };
type Required<T> = { [K in keyof T]-?: T[K] };
type Readonly<T> = { readonly [K in keyof T]: T[K] };
type Pick<T, K extends keyof T> = { [P in K]: T[P] };
type Omit<T, K extends keyof any> = Pick<T, Exclude<keyof T, K>>;

// Custom: Make specific keys required
type RequireKeys<T, K extends keyof T> = Omit<T, K> & Required<Pick<T, K>>;

interface User {
  id: string;
  name?: string;
  email?: string;
  age?: number;
}

type UserWithEmail = RequireKeys<User, 'email'>;
// { id: string; name?: string; email: string; age?: number }

// Custom: Nullable all properties
type Nullable<T> = { [K in keyof T]: T[K] | null };

// Custom: Deep partial
type DeepPartial<T> = T extends object
  ? { [K in keyof T]?: DeepPartial<T[K]> }
  : T;

// Custom: Mutable (remove readonly)
type Mutable<T> = { -readonly [K in keyof T]: T[K] };

// Custom: Form state from entity
type FormState<T> = {
  values: T;
  errors: { [K in keyof T]?: string };
  touched: { [K in keyof T]?: boolean };
  dirty: { [K in keyof T]?: boolean };
};

// Custom: Prefix all keys
type Prefixed<T, P extends string> = {
  [K in keyof T as `${P}${Capitalize<string & K>}`]: T[K];
};

type UserPrefixed = Prefixed<User, 'user'>;
// { userId: string; userName?: string; userEmail?: string; userAge?: number }

// Custom: Filter keys by type
type KeysOfType<T, V> = {
  [K in keyof T]: T[K] extends V ? K : never;
}[keyof T];

type StringKeys = KeysOfType<User, string | undefined>;
// 'id' | 'name' | 'email'
```

TS.1.8 Conditional Types with Infer
- Use when: Extracting types from generic types
- Context: Use `infer` to capture type variables in conditional types
```typescript
// Extract return type
type ReturnType<T> = T extends (...args: any[]) => infer R ? R : never;

// Extract parameter types
type Parameters<T> = T extends (...args: infer P) => any ? P : never;

// Extract element type from array
type ElementType<T> = T extends (infer E)[] ? E : T extends readonly (infer E)[] ? E : never;

// Extract promise value
type Awaited<T> = T extends Promise<infer V> ? Awaited<V> : T;

// Extract props from React component
type PropsOf<T> = T extends React.ComponentType<infer P> ? P : never;

// Extract second parameter
type SecondParam<T> = T extends (a: any, b: infer B, ...args: any[]) => any ? B : never;

// Practical: Extract API response data type
type APIResponse<T> = {
  data: T;
  status: number;
  message: string;
  timestamp: Date;
};

type ExtractData<T> = T extends APIResponse<infer D> ? D : never;

type UserResponse = APIResponse<User>;
type UserData = ExtractData<UserResponse>; // User

// Extract function argument at position
type ArgumentAtPosition<T, N extends number> = 
  T extends (...args: infer A) => any
    ? A[N]
    : never;

function example(a: string, b: number, c: boolean): void {}
type SecondArg = ArgumentAtPosition<typeof example, 1>; // number

// Unpack nested types
type UnpackPromise<T> = T extends Promise<infer U> ? UnpackPromise<U> : T;
type UnpackArray<T> = T extends (infer U)[] ? UnpackArray<U> : T;

type Nested = Promise<Promise<string[]>>;
type Unpacked = UnpackPromise<Nested>; // string[]
```

TS.1.9 Function Overloads
- Use when: Function has different return types based on input
- Context: Define overload signatures before implementation
```typescript
// Overloaded function
function createElement(tag: 'input'): HTMLInputElement;
function createElement(tag: 'div'): HTMLDivElement;
function createElement(tag: 'span'): HTMLSpanElement;
function createElement(tag: 'a'): HTMLAnchorElement;
function createElement(tag: string): HTMLElement;
function createElement(tag: string): HTMLElement {
  return document.createElement(tag);
}

const input = createElement('input'); // HTMLInputElement
const div = createElement('div');     // HTMLDivElement
const link = createElement('a');      // HTMLAnchorElement

// Overloaded with different parameter shapes
function parseValue(value: string): string;
function parseValue(value: number): number;
function parseValue(value: string | number): string | number {
  return value;
}

// Overloaded async function
async function fetchData(id: string): Promise<User>;
async function fetchData(ids: string[]): Promise<User[]>;
async function fetchData(idOrIds: string | string[]): Promise<User | User[]> {
  if (Array.isArray(idOrIds)) {
    return Promise.all(idOrIds.map(id => fetchUserById(id)));
  }
  return fetchUserById(idOrIds);
}

// Class method overloads
class EventEmitter<Events extends Record<string, unknown[]>> {
  on<K extends keyof Events>(
    event: K,
    handler: (...args: Events[K]) => void
  ): this;
  on(event: string, handler: (...args: unknown[]) => void): this {
    // Implementation
    return this;
  }

  emit<K extends keyof Events>(event: K, ...args: Events[K]): boolean;
  emit(event: string, ...args: unknown[]): boolean {
    // Implementation
    return true;
  }
}

// Usage
type AppEvents = {
  userLogin: [userId: string, timestamp: Date];
  orderPlaced: [orderId: string, total: number];
  error: [error: Error];
};

const emitter = new EventEmitter<AppEvents>();
emitter.on('userLogin', (userId, timestamp) => {
  // userId: string, timestamp: Date
});
emitter.on('orderPlaced', (orderId, total) => {
  // orderId: string, total: number
});
```

TS.1.10 Satisfies Operator
- Use when: Validating type compatibility while preserving literal types
- Context: Use `satisfies` when you need both validation AND inference
```typescript
// Problem: Type annotation loses literal types
const routes1: Record<string, string> = {
  home: '/',
  users: '/users',
  settings: '/settings',
};
// routes1.home is string, not '/'

// Solution: satisfies preserves literals
const routes2 = {
  home: '/',
  users: '/users',
  settings: '/settings',
} satisfies Record<string, string>;
// routes2.home is '/' (literal type preserved)
// AND validated against Record<string, string>

// Practical: Config validation with preserved types
type ColorConfig = Record<string, [number, number, number]>;

const colors = {
  red: [255, 0, 0],
  green: [0, 255, 0],
  blue: [0, 0, 255],
  // yellow: [255, 255], // ❌ Error: wrong tuple length
} satisfies ColorConfig;

colors.red; // type: [number, number, number] but also ensures validity

// Theme configuration
type Theme = {
  colors: Record<string, string>;
  spacing: Record<string, number>;
  breakpoints: Record<string, string>;
};

const theme = {
  colors: {
    primary: '#007bff',
    secondary: '#6c757d',
    success: '#28a745',
  },
  spacing: {
    xs: 4,
    sm: 8,
    md: 16,
    lg: 24,
    xl: 32,
  },
  breakpoints: {
    sm: '640px',
    md: '768px',
    lg: '1024px',
    xl: '1280px',
  },
} satisfies Theme;

// theme.colors.primary is '#007bff' (literal preserved)
// theme.spacing.md is 16 (literal preserved)

// API routes with inference
const apiRoutes = {
  users: {
    list: '/api/users',
    get: '/api/users/:id',
    create: '/api/users',
  },
  orders: {
    list: '/api/orders',
    get: '/api/orders/:id',
  },
} satisfies Record<string, Record<string, string>>;
```

---

## TS.2 ERROR HANDLING PATTERNS

TS.2.1 Result Type Pattern
- Use when: Representing operations that can fail without exceptions
- Context: Create a discriminated union for success/failure
```typescript
// Result type definition
type Result<T, E = Error> =
  | { ok: true; value: T }
  | { ok: false; error: E };

// Namespace for Result operations
namespace Result {
  export function ok<T>(value: T): Result<T, never> {
    return { ok: true, value };
  }

  export function err<E>(error: E): Result<never, E> {
    return { ok: false, error };
  }

  export function map<T, U, E>(
    result: Result<T, E>,
    fn: (value: T) => U
  ): Result<U, E> {
    if (result.ok) {
      return { ok: true, value: fn(result.value) };
    }
    return result;
  }

  export function flatMap<T, U, E>(
    result: Result<T, E>,
    fn: (value: T) => Result<U, E>
  ): Result<U, E> {
    if (result.ok) {
      return fn(result.value);
    }
    return result;
  }

  export function unwrapOr<T, E>(result: Result<T, E>, defaultValue: T): T {
    return result.ok ? result.value : defaultValue;
  }

  export function isOk<T, E>(result: Result<T, E>): result is { ok: true; value: T } {
    return result.ok;
  }

  export function isErr<T, E>(result: Result<T, E>): result is { ok: false; error: E } {
    return !result.ok;
  }
}

// Usage
type UserError = 
  | { code: 'NOT_FOUND'; userId: string }
  | { code: 'VALIDATION'; field: string; message: string }
  | { code: 'UNAUTHORIZED' };

async function createUser(data: CreateUserDTO): Promise<Result<User, UserError>> {
  if (!data.email.includes('@')) {
    return Result.err({ 
      code: 'VALIDATION', 
      field: 'email', 
      message: 'Invalid email format' 
    });
  }

  const existingUser = await userRepo.findByEmail(data.email);
  if (existingUser) {
    return Result.err({ 
      code: 'VALIDATION', 
      field: 'email', 
      message: 'Email already exists' 
    });
  }

  const user = await userRepo.save(User.create(data));
  return Result.ok(user);
}

// Consuming Result
const result = await createUser({ email: 'test@example.com', name: 'Test' });

if (result.ok) {
  console.log('Created user:', result.value.id);
} else {
  switch (result.error.code) {
    case 'NOT_FOUND':
      console.error(`User not found: ${result.error.userId}`);
      break;
    case 'VALIDATION':
      console.error(`Validation error on ${result.error.field}: ${result.error.message}`);
      break;
    case 'UNAUTHORIZED':
      console.error('Unauthorized');
      break;
  }
}
```
- Avoid/Anti-pattern: Throwing exceptions for expected failures
```typescript
// ❌ Bad: Using exceptions for control flow
async function createUser(data: CreateUserDTO): Promise<User> {
  if (!data.email.includes('@')) {
    throw new Error('Invalid email'); // Caller must remember to catch
  }
  // ...
}
```

TS.2.2 Zod Schema Validation
- Use when: Validating external data at runtime with type inference
- Context: Define schema once, get validation AND TypeScript types
```typescript
import { z } from 'zod';

// Define schemas
const EmailSchema = z.string().email().toLowerCase().trim();
const PasswordSchema = z.string()
  .min(8, 'Password must be at least 8 characters')
  .regex(/[A-Z]/, 'Password must contain uppercase letter')
  .regex(/[a-z]/, 'Password must contain lowercase letter')
  .regex(/[0-9]/, 'Password must contain number');

const CreateUserSchema = z.object({
  email: EmailSchema,
  password: PasswordSchema,
  name: z.string().min(1).max(100),
  age: z.number().int().min(0).max(150).optional(),
  role: z.enum(['user', 'admin', 'moderator']).default('user'),
  preferences: z.object({
    newsletter: z.boolean().default(false),
    theme: z.enum(['light', 'dark']).default('light'),
  }).optional(),
});

// Infer TypeScript type from schema
type CreateUserDTO = z.infer<typeof CreateUserSchema>;
// {
//   email: string;
//   password: string;
//   name: string;
//   age?: number;
//   role: 'user' | 'admin' | 'moderator';
//   preferences?: { newsletter: boolean; theme: 'light' | 'dark' };
// }

// Validation functions
function parseCreateUser(data: unknown): Result<CreateUserDTO, z.ZodError> {
  const result = CreateUserSchema.safeParse(data);
  if (result.success) {
    return { ok: true, value: result.data };
  }
  return { ok: false, error: result.error };
}

// Express middleware
function validateBody<T extends z.ZodSchema>(schema: T) {
  return (req: Request, res: Response, next: NextFunction) => {
    const result = schema.safeParse(req.body);
    if (!result.success) {
      return res.status(400).json({
        error: 'Validation failed',
        details: result.error.flatten(),
      });
    }
    req.body = result.data;
    next();
  };
}

// Usage in route
app.post('/users', validateBody(CreateUserSchema), async (req, res) => {
  const userData: CreateUserDTO = req.body; // Fully typed and validated
  // ...
});

// Composing schemas
const UpdateUserSchema = CreateUserSchema.partial().omit({ password: true });
const UserResponseSchema = CreateUserSchema.omit({ password: true }).extend({
  id: z.string().uuid(),
  createdAt: z.date(),
  updatedAt: z.date(),
});
```

TS.2.3 Try-Catch Wrapper for Async
- Use when: Converting try/catch to tuple result pattern
- Context: Create utility for consistent async error handling
```typescript
// Tuple-based error handling
type AsyncResult<T> = Promise<[T, null] | [null, Error]>;

async function tryCatch<T>(promise: Promise<T>): AsyncResult<T> {
  try {
    const data = await promise;
    return [data, null];
  } catch (error) {
    const err = error instanceof Error ? error : new Error(String(error));
    return [null, err];
  }
}

// Usage
async function handleUserRequest(userId: string) {
  const [user, error] = await tryCatch(userService.findById(userId));
  
  if (error) {
    logger.error('Failed to fetch user', { userId, error: error.message });
    return { error: 'User fetch failed' };
  }
  
  // TypeScript knows user is not null here
  return { user };
}

// Multiple operations
async function processOrder(orderId: string) {
  const [order, orderError] = await tryCatch(orderService.findById(orderId));
  if (orderError) return { error: 'Order not found' };

  const [inventory, inventoryError] = await tryCatch(
    inventoryService.checkAvailability(order.items)
  );
  if (inventoryError) return { error: 'Inventory check failed' };

  const [payment, paymentError] = await tryCatch(
    paymentService.process(order.total)
  );
  if (paymentError) return { error: 'Payment failed' };

  return { success: true, order, payment };
}

// With typed errors
type TypedAsyncResult<T, E> = Promise<
  { success: true; data: T } | { success: false; error: E }
>;

async function tryCatchTyped<T, E>(
  promise: Promise<T>,
  errorMapper: (error: unknown) => E
): TypedAsyncResult<T, E> {
  try {
    const data = await promise;
    return { success: true, data };
  } catch (error) {
    return { success: false, error: errorMapper(error) };
  }
}
```

TS.2.4 Custom Error Classes
- Use when: Creating domain-specific errors with structured data
- Context: Extend Error class with additional properties
```typescript
// Base application error
abstract class AppError extends Error {
  abstract readonly code: string;
  abstract readonly statusCode: number;
  readonly timestamp: Date;
  readonly context?: Record<string, unknown>;

  constructor(message: string, context?: Record<string, unknown>) {
    super(message);
    this.name = this.constructor.name;
    this.timestamp = new Date();
    this.context = context;
    Error.captureStackTrace(this, this.constructor);
  }

  toJSON() {
    return {
      code: this.code,
      message: this.message,
      timestamp: this.timestamp.toISOString(),
      context: this.context,
    };
  }
}

// Specific error types
class ValidationError extends AppError {
  readonly code = 'VALIDATION_ERROR';
  readonly statusCode = 400;
  readonly field: string;

  constructor(field: string, message: string) {
    super(message, { field });
    this.field = field;
  }
}

class NotFoundError extends AppError {
  readonly code = 'NOT_FOUND';
  readonly statusCode = 404;
  readonly resource: string;
  readonly id: string;

  constructor(resource: string, id: string) {
    super(`${resource} with id '${id}' not found`, { resource, id });
    this.resource = resource;
    this.id = id;
  }
}

class UnauthorizedError extends AppError {
  readonly code = 'UNAUTHORIZED';
  readonly statusCode = 401;

  constructor(message = 'Authentication required') {
    super(message);
  }
}

class ForbiddenError extends AppError {
  readonly code = 'FORBIDDEN';
  readonly statusCode = 403;
  readonly requiredPermission?: string;

  constructor(message = 'Access denied', requiredPermission?: string) {
    super(message, { requiredPermission });
    this.requiredPermission = requiredPermission;
  }
}

class ConflictError extends AppError {
  readonly code = 'CONFLICT';
  readonly statusCode = 409;

  constructor(message: string) {
    super(message);
  }
}

// Type guard
function isAppError(error: unknown): error is AppError {
  return error instanceof AppError;
}

// Error handler middleware
function errorHandler(
  err: Error,
  req: Request,
  res: Response,
  next: NextFunction
) {
  if (isAppError(err)) {
    return res.status(err.statusCode).json(err.toJSON());
  }

  // Log unexpected errors
  logger.error('Unexpected error', { error: err.message, stack: err.stack });

  return res.status(500).json({
    code: 'INTERNAL_ERROR',
    message: 'An unexpected error occurred',
  });
}

// Usage
async function getUser(id: string): Promise<User> {
  const user = await userRepository.findById(id);
  if (!user) {
    throw new NotFoundError('User', id);
  }
  return user;
}
```

TS.2.5 neverthrow Library Pattern
- Use when: Want a full-featured Result type with railway-oriented programming
- Context: Use neverthrow for composable error handling
```typescript
import { Result, ok, err, ResultAsync } from 'neverthrow';

// Define error types
type UserError =
  | { type: 'NOT_FOUND'; userId: string }
  | { type: 'VALIDATION'; errors: string[] }
  | { type: 'DATABASE'; message: string };

// Synchronous Result
function validateEmail(email: string): Result<string, UserError> {
  if (!email.includes('@')) {
    return err({ type: 'VALIDATION', errors: ['Invalid email format'] });
  }
  return ok(email.toLowerCase().trim());
}

function validatePassword(password: string): Result<string, UserError> {
  const errors: string[] = [];
  if (password.length < 8) errors.push('Password too short');
  if (!/[A-Z]/.test(password)) errors.push('Missing uppercase');
  if (!/[0-9]/.test(password)) errors.push('Missing number');
  
  if (errors.length > 0) {
    return err({ type: 'VALIDATION', errors });
  }
  return ok(password);
}

// Chaining Results
function validateUserInput(input: { email: string; password: string }) {
  return validateEmail(input.email)
    .andThen(email => 
      validatePassword(input.password)
        .map(password => ({ email, password }))
    );
}

// Async Results
function findUserById(id: string): ResultAsync<User, UserError> {
  return ResultAsync.fromPromise(
    userRepository.findById(id),
    (error) => ({ type: 'DATABASE', message: String(error) })
  ).andThen(user => 
    user ? ok(user) : err({ type: 'NOT_FOUND', userId: id })
  );
}

// Combining multiple Results
function createUser(input: CreateUserDTO): ResultAsync<User, UserError> {
  return validateUserInput(input)
    .asyncAndThen(validated =>
      ResultAsync.fromPromise(
        userRepository.save(User.create(validated)),
        (error) => ({ type: 'DATABASE', message: String(error) })
      )
    );
}

// Pattern matching on result
const result = await createUser({ email: 'test@example.com', password: 'Test1234' });

result.match(
  (user) => console.log('Created user:', user.id),
  (error) => {
    switch (error.type) {
      case 'NOT_FOUND':
        console.error(`User ${error.userId} not found`);
        break;
      case 'VALIDATION':
        console.error('Validation errors:', error.errors.join(', '));
        break;
      case 'DATABASE':
        console.error('Database error:', error.message);
        break;
    }
  }
);
```

---

## TS.3 ASYNC PATTERNS

TS.3.1 Promise.allSettled for Partial Failures
- Use when: Running multiple operations where some can fail independently
- Context: Use allSettled to get results of all promises regardless of failures
```typescript
interface User {
  id: string;
  name: string;
  email: string;
}

async function fetchUsers(ids: string[]): Promise<Map<string, User>> {
  const results = await Promise.allSettled(
    ids.map(id => fetchUser(id))
  );
  
  const users = new Map<string, User>();
  const errors: Array<{ id: string; error: string }> = [];
  
  results.forEach((result, index) => {
    const id = ids[index];
    if (result.status === 'fulfilled') {
      users.set(id, result.value);
    } else {
      errors.push({ id, error: result.reason.message });
    }
  });
  
  if (errors.length > 0) {
    logger.warn('Some users failed to fetch', { errors });
  }
  
  return users;
}

// Typed wrapper for allSettled
type SettledResult<T> =
  | { status: 'fulfilled'; value: T; index: number }
  | { status: 'rejected'; reason: Error; index: number };

async function allSettledWithIndex<T>(
  promises: Promise<T>[]
): Promise<SettledResult<T>[]> {
  const results = await Promise.allSettled(promises);
  return results.map((result, index) => {
    if (result.status === 'fulfilled') {
      return { status: 'fulfilled', value: result.value, index };
    }
    return {
      status: 'rejected',
      reason: result.reason instanceof Error 
        ? result.reason 
        : new Error(String(result.reason)),
      index,
    };
  });
}

// Partition results
function partitionSettled<T>(
  results: SettledResult<T>[]
): { successes: T[]; failures: Array<{ error: Error; index: number }> } {
  const successes: T[] = [];
  const failures: Array<{ error: Error; index: number }> = [];
  
  for (const result of results) {
    if (result.status === 'fulfilled') {
      successes.push(result.value);
    } else {
      failures.push({ error: result.reason, index: result.index });
    }
  }
  
  return { successes, failures };
}
```

TS.3.2 Retry with Exponential Backoff
- Use when: Handling transient failures in network requests
- Context: Implement retry logic with configurable backoff
```typescript
interface RetryOptions {
  maxAttempts: number;
  baseDelayMs: number;
  maxDelayMs: number;
  shouldRetry?: (error: Error, attempt: number) => boolean;
  onRetry?: (error: Error, attempt: number, delayMs: number) => void;
}

const defaultRetryOptions: RetryOptions = {
  maxAttempts: 3,
  baseDelayMs: 1000,
  maxDelayMs: 30000,
  shouldRetry: (error) => {
    // Retry on network errors or 5xx status codes
    if (error.name === 'NetworkError') return true;
    if ('statusCode' in error) {
      const statusCode = (error as any).statusCode;
      return statusCode >= 500 && statusCode < 600;
    }
    return false;
  },
};

async function withRetry<T>(
  fn: () => Promise<T>,
  options: Partial<RetryOptions> = {}
): Promise<T> {
  const opts = { ...defaultRetryOptions, ...options };
  let lastError: Error;
  
  for (let attempt = 1; attempt <= opts.maxAttempts; attempt++) {
    try {
      return await fn();
    } catch (error) {
      lastError = error instanceof Error ? error : new Error(String(error));
      
      const isLastAttempt = attempt === opts.maxAttempts;
      const shouldRetry = opts.shouldRetry?.(lastError, attempt) ?? true;
      
      if (isLastAttempt || !shouldRetry) {
        throw lastError;
      }
      
      // Calculate delay with exponential backoff and jitter
      const exponentialDelay = opts.baseDelayMs * Math.pow(2, attempt - 1);
      const jitter = Math.random() * 0.1 * exponentialDelay;
      const delayMs = Math.min(exponentialDelay + jitter, opts.maxDelayMs);
      
      opts.onRetry?.(lastError, attempt, delayMs);
      
      await sleep(delayMs);
    }
  }
  
  throw lastError!;
}

function sleep(ms: number): Promise<void> {
  return new Promise(resolve => setTimeout(resolve, ms));
}

// Usage
const user = await withRetry(
  () => fetchUser(userId),
  {
    maxAttempts: 5,
    baseDelayMs: 500,
    onRetry: (error, attempt, delay) => {
      logger.warn(`Retry attempt ${attempt}, waiting ${delay}ms`, { error: error.message });
    },
  }
);

// Decorator version
function Retryable(options: Partial<RetryOptions> = {}) {
  return function (
    target: any,
    propertyKey: string,
    descriptor: PropertyDescriptor
  ) {
    const originalMethod = descriptor.value;
    
    descriptor.value = async function (...args: any[]) {
      return withRetry(() => originalMethod.apply(this, args), options);
    };
    
    return descriptor;
  };
}

// Usage with decorator
class UserService {
  @Retryable({ maxAttempts: 3, baseDelayMs: 1000 })
  async fetchUser(id: string): Promise<User> {
    return this.httpClient.get(`/users/${id}`);
  }
}
```

TS.3.3 Debounce and Throttle
- Use when: Limiting rate of function execution
- Context: Create type-safe debounce/throttle utilities
```typescript
type AnyFunction = (...args: any[]) => any;

interface DebouncedFunction<T extends AnyFunction> {
  (...args: Parameters<T>): void;
  cancel: () => void;
  flush: () => void;
}

function debounce<T extends AnyFunction>(
  fn: T,
  wait: number
): DebouncedFunction<T> {
  let timeoutId: ReturnType<typeof setTimeout> | null = null;
  let lastArgs: Parameters<T> | null = null;
  
  const debounced = (...args: Parameters<T>) => {
    lastArgs = args;
    
    if (timeoutId) {
      clearTimeout(timeoutId);
    }
    
    timeoutId = setTimeout(() => {
      fn(...args);
      timeoutId = null;
      lastArgs = null;
    }, wait);
  };
  
  debounced.cancel = () => {
    if (timeoutId) {
      clearTimeout(timeoutId);
      timeoutId = null;
      lastArgs = null;
    }
  };
  
  debounced.flush = () => {
    if (timeoutId && lastArgs) {
      clearTimeout(timeoutId);
      fn(...lastArgs);
      timeoutId = null;
      lastArgs = null;
    }
  };
  
  return debounced;
}

interface ThrottledFunction<T extends AnyFunction> {
  (...args: Parameters<T>): void;
  cancel: () => void;
}

function throttle<T extends AnyFunction>(
  fn: T,
  limit: number
): ThrottledFunction<T> {
  let inThrottle = false;
  let lastArgs: Parameters<T> | null = null;
  
  const throttled = (...args: Parameters<T>) => {
    if (!inThrottle) {
      fn(...args);
      inThrottle = true;
      
      setTimeout(() => {
        inThrottle = false;
        if (lastArgs) {
          throttled(...lastArgs);
          lastArgs = null;
        }
      }, limit);
    } else {
      lastArgs = args;
    }
  };
  
  throttled.cancel = () => {
    inThrottle = false;
    lastArgs = null;
  };
  
  return throttled;
}

// Usage
const debouncedSearch = debounce((query: string) => {
  console.log('Searching:', query);
  performSearch(query);
}, 300);

// In React
function SearchInput() {
  const handleChange = useMemo(
    () => debounce((value: string) => {
      setSearchResults(value);
    }, 300),
    []
  );
  
  useEffect(() => {
    return () => handleChange.cancel();
  }, [handleChange]);
  
  return <input onChange={(e) => handleChange(e.target.value)} />;
}
```

TS.3.4 Async Queue with Concurrency Control
- Use when: Processing many async tasks with limited parallelism
- Context: Implement a queue that limits concurrent operations
```typescript
interface QueueOptions {
  concurrency: number;
  timeout?: number;
}

class AsyncQueue<T, R> {
  private queue: Array<{
    item: T;
    resolve: (value: R) => void;
    reject: (error: Error) => void;
  }> = [];
  private running = 0;
  private readonly concurrency: number;
  private readonly timeout?: number;
  private readonly processor: (item: T) => Promise<R>;

  constructor(
    processor: (item: T) => Promise<R>,
    options: QueueOptions
  ) {
    this.processor = processor;
    this.concurrency = options.concurrency;
    this.timeout = options.timeout;
  }

  add(item: T): Promise<R> {
    return new Promise((resolve, reject) => {
      this.queue.push({ item, resolve, reject });
      this.processNext();
    });
  }

  addAll(items: T[]): Promise<R[]> {
    return Promise.all(items.map(item => this.add(item)));
  }

  private async processNext(): Promise<void> {
    if (this.running >= this.concurrency || this.queue.length === 0) {
      return;
    }

    const { item, resolve, reject } = this.queue.shift()!;
    this.running++;

    try {
      const promise = this.processor(item);
      const result = this.timeout
        ? await this.withTimeout(promise, this.timeout)
        : await promise;
      resolve(result);
    } catch (error) {
      reject(error instanceof Error ? error : new Error(String(error)));
    } finally {
      this.running--;
      this.processNext();
    }
  }

  private withTimeout<T>(promise: Promise<T>, ms: number): Promise<T> {
    return Promise.race([
      promise,
      new Promise<never>((_, reject) =>
        setTimeout(() => reject(new Error(`Timeout after ${ms}ms`)), ms)
      ),
    ]);
  }

  get pending(): number {
    return this.queue.length;
  }

  get active(): number {
    return this.running;
  }
}

// Usage
const queue = new AsyncQueue<string, User>(
  (userId) => fetchUser(userId),
  { concurrency: 5, timeout: 10000 }
);

// Process many users with max 5 concurrent requests
const userIds = ['u1', 'u2', 'u3', /* ... hundreds more */];
const users = await queue.addAll(userIds);

// P-limit style
import pLimit from 'p-limit';

const limit = pLimit(5);

const users = await Promise.all(
  userIds.map(id => limit(() => fetchUser(id)))
);
```

TS.3.5 Cancellable Promises with AbortController
- Use when: Need to cancel in-flight async operations
- Context: Use AbortController for cancellation
```typescript
function makeCancellable<T>(
  promise: Promise<T>,
  signal: AbortSignal
): Promise<T> {
  return new Promise((resolve, reject) => {
    // Check if already aborted
    if (signal.aborted) {
      reject(new DOMException('Aborted', 'AbortError'));
      return;
    }

    // Listen for abort
    const abortHandler = () => {
      reject(new DOMException('Aborted', 'AbortError'));
    };
    signal.addEventListener('abort', abortHandler);

    promise
      .then(resolve)
      .catch(reject)
      .finally(() => {
        signal.removeEventListener('abort', abortHandler);
      });
  });
}

// Cancellable fetch wrapper
async function cancellableFetch<T>(
  url: string,
  options: RequestInit & { signal?: AbortSignal } = {}
): Promise<T> {
  const response = await fetch(url, options);
  
  if (!response.ok) {
    throw new Error(`HTTP error! status: ${response.status}`);
  }
  
  return response.json();
}

// Usage with timeout
async function fetchWithTimeout<T>(
  url: string,
  timeoutMs: number
): Promise<T> {
  const controller = new AbortController();
  const timeoutId = setTimeout(() => controller.abort(), timeoutMs);
  
  try {
    return await cancellableFetch<T>(url, { signal: controller.signal });
  } finally {
    clearTimeout(timeoutId);
  }
}

// React hook for cancellable requests
function useCancellableRequest<T>() {
  const controllerRef = useRef<AbortController | null>(null);
  
  const execute = useCallback(async (
    requestFn: (signal: AbortSignal) => Promise<T>
  ): Promise<T> => {
    // Cancel previous request
    controllerRef.current?.abort();
    
    // Create new controller
    controllerRef.current = new AbortController();
    
    return requestFn(controllerRef.current.signal);
  }, []);
  
  const cancel = useCallback(() => {
    controllerRef.current?.abort();
  }, []);
  
  // Cleanup on unmount
  useEffect(() => {
    return () => {
      controllerRef.current?.abort();
    };
  }, []);
  
  return { execute, cancel };
}

// Usage in component
function UserProfile({ userId }: { userId: string }) {
  const { execute, cancel } = useCancellableRequest<User>();
  const [user, setUser] = useState<User | null>(null);
  
  useEffect(() => {
    execute(async (signal) => {
      const data = await cancellableFetch<User>(
        `/api/users/${userId}`,
        { signal }
      );
      setUser(data);
      return data;
    }).catch((error) => {
      if (error.name !== 'AbortError') {
        console.error('Failed to fetch user:', error);
      }
    });
    
    return () => cancel();
  }, [userId, execute, cancel]);
  
  return user ? <div>{user.name}</div> : <div>Loading...</div>;
}
```

---

## TS.4 REACT PATTERNS

TS.4.1 Compound Components with Context
- Use when: Building components that work together as a cohesive unit
- Context: Use Context to share state between related components
```typescript
import { createContext, useContext, useState, ReactNode } from 'react';

// Types
interface AccordionContextValue {
  activeItems: Set<string>;
  toggle: (id: string) => void;
  isExpanded: (id: string) => boolean;
}

interface AccordionProps {
  children: ReactNode;
  multiple?: boolean;
  defaultExpanded?: string[];
}

interface AccordionItemProps {
  id: string;
  children: ReactNode;
}

interface AccordionTriggerProps {
  children: ReactNode;
}

interface AccordionContentProps {
  children: ReactNode;
}

// Context
const AccordionContext = createContext<AccordionContextValue | null>(null);
const AccordionItemContext = createContext<string | null>(null);

function useAccordion(): AccordionContextValue {
  const context = useContext(AccordionContext);
  if (!context) {
    throw new Error('useAccordion must be used within Accordion');
  }
  return context;
}

function useAccordionItem(): string {
  const context = useContext(AccordionItemContext);
  if (!context) {
    throw new Error('useAccordionItem must be used within Accordion.Item');
  }
  return context;
}

// Components
function Accordion({ children, multiple = false, defaultExpanded = [] }: AccordionProps) {
  const [activeItems, setActiveItems] = useState<Set<string>>(
    new Set(defaultExpanded)
  );

  const toggle = (id: string) => {
    setActiveItems(prev => {
      const next = new Set(prev);
      if (next.has(id)) {
        next.delete(id);
      } else {
        if (!multiple) {
          next.clear();
        }
        next.add(id);
      }
      return next;
    });
  };

  const isExpanded = (id: string) => activeItems.has(id);

  return (
    <AccordionContext.Provider value={{ activeItems, toggle, isExpanded }}>
      <div className="accordion">{children}</div>
    </AccordionContext.Provider>
  );
}

function AccordionItem({ id, children }: AccordionItemProps) {
  return (
    <AccordionItemContext.Provider value={id}>
      <div className="accordion-item">{children}</div>
    </AccordionItemContext.Provider>
  );
}

function AccordionTrigger({ children }: AccordionTriggerProps) {
  const { toggle, isExpanded } = useAccordion();
  const id = useAccordionItem();

  return (
    <button
      className="accordion-trigger"
      onClick={() => toggle(id)}
      aria-expanded={isExpanded(id)}
    >
      {children}
    </button>
  );
}

function AccordionContent({ children }: AccordionContentProps) {
  const { isExpanded } = useAccordion();
  const id = useAccordionItem();

  if (!isExpanded(id)) return null;

  return <div className="accordion-content">{children}</div>;
}

// Attach sub-components
Accordion.Item = AccordionItem;
Accordion.Trigger = AccordionTrigger;
Accordion.Content = AccordionContent;

// Usage
function FAQ() {
  return (
    <Accordion multiple defaultExpanded={['item-1']}>
      <Accordion.Item id="item-1">
        <Accordion.Trigger>What is TypeScript?</Accordion.Trigger>
        <Accordion.Content>
          TypeScript is a typed superset of JavaScript...
        </Accordion.Content>
      </Accordion.Item>
      <Accordion.Item id="item-2">
        <Accordion.Trigger>Why use TypeScript?</Accordion.Trigger>
        <Accordion.Content>
          TypeScript provides type safety and better tooling...
        </Accordion.Content>
      </Accordion.Item>
    </Accordion>
  );
}
```

TS.4.2 Custom Hooks with Proper TypeScript
- Use when: Extracting reusable stateful logic
- Context: Define explicit return types and proper generics
```typescript
import { useState, useEffect, useCallback, useRef } from 'react';

// useAsync hook
interface UseAsyncState<T> {
  data: T | null;
  error: Error | null;
  isLoading: boolean;
  isSuccess: boolean;
  isError: boolean;
}

interface UseAsyncReturn<T, Args extends unknown[]> extends UseAsyncState<T> {
  execute: (...args: Args) => Promise<T | null>;
  reset: () => void;
}

function useAsync<T, Args extends unknown[] = []>(
  asyncFn: (...args: Args) => Promise<T>
): UseAsyncReturn<T, Args> {
  const [state, setState] = useState<UseAsyncState<T>>({
    data: null,
    error: null,
    isLoading: false,
    isSuccess: false,
    isError: false,
  });

  // Track mounted state to prevent state updates after unmount
  const isMounted = useRef(true);

  useEffect(() => {
    return () => {
      isMounted.current = false;
    };
  }, []);

  const execute = useCallback(
    async (...args: Args): Promise<T | null> => {
      setState(prev => ({
        ...prev,
        isLoading: true,
        isSuccess: false,
        isError: false,
      }));

      try {
        const data = await asyncFn(...args);
        if (isMounted.current) {
          setState({
            data,
            error: null,
            isLoading: false,
            isSuccess: true,
            isError: false,
          });
        }
        return data;
      } catch (error) {
        const err = error instanceof Error ? error : new Error(String(error));
        if (isMounted.current) {
          setState({
            data: null,
            error: err,
            isLoading: false,
            isSuccess: false,
            isError: true,
          });
        }
        return null;
      }
    },
    [asyncFn]
  );

  const reset = useCallback(() => {
    setState({
      data: null,
      error: null,
      isLoading: false,
      isSuccess: false,
      isError: false,
    });
  }, []);

  return { ...state, execute, reset };
}

// useLocalStorage hook
function useLocalStorage<T>(
  key: string,
  initialValue: T
): [T, (value: T | ((prev: T) => T)) => void, () => void] {
  const [storedValue, setStoredValue] = useState<T>(() => {
    if (typeof window === 'undefined') return initialValue;
    
    try {
      const item = window.localStorage.getItem(key);
      return item ? JSON.parse(item) : initialValue;
    } catch {
      return initialValue;
    }
  });

  const setValue = useCallback(
    (value: T | ((prev: T) => T)) => {
      try {
        const valueToStore = value instanceof Function ? value(storedValue) : value;
        setStoredValue(valueToStore);
        window.localStorage.setItem(key, JSON.stringify(valueToStore));
      } catch (error) {
        console.error('Error saving to localStorage:', error);
      }
    },
    [key, storedValue]
  );

  const removeValue = useCallback(() => {
    try {
      window.localStorage.removeItem(key);
      setStoredValue(initialValue);
    } catch (error) {
      console.error('Error removing from localStorage:', error);
    }
  }, [key, initialValue]);

  return [storedValue, setValue, removeValue];
}

// useDebounce hook
function useDebounce<T>(value: T, delay: number): T {
  const [debouncedValue, setDebouncedValue] = useState<T>(value);

  useEffect(() => {
    const timer = setTimeout(() => setDebouncedValue(value), delay);
    return () => clearTimeout(timer);
  }, [value, delay]);

  return debouncedValue;
}

// usePrevious hook
function usePrevious<T>(value: T): T | undefined {
  const ref = useRef<T>();
  
  useEffect(() => {
    ref.current = value;
  }, [value]);
  
  return ref.current;
}

// Usage
function UserSearch() {
  const [query, setQuery] = useState('');
  const debouncedQuery = useDebounce(query, 300);
  const { data: users, isLoading, execute } = useAsync(searchUsers);

  useEffect(() => {
    if (debouncedQuery) {
      execute(debouncedQuery);
    }
  }, [debouncedQuery, execute]);

  return (
    <div>
      <input
        value={query}
        onChange={(e) => setQuery(e.target.value)}
        placeholder="Search users..."
      />
      {isLoading ? (
        <div>Loading...</div>
      ) : (
        <ul>
          {users?.map(user => (
            <li key={user.id}>{user.name}</li>
          ))}
        </ul>
      )}
    </div>
  );
}
```

TS.4.3 TanStack Query (React Query) Patterns
- Use when: Managing server state in React applications
- Context: Use useQuery/useMutation with proper typing
```typescript
import {
  useQuery,
  useMutation,
  useQueryClient,
  QueryClient,
  QueryClientProvider,
} from '@tanstack/react-query';

// API types
interface User {
  id: string;
  name: string;
  email: string;
}

interface CreateUserDTO {
  name: string;
  email: string;
}

// Query keys factory
const userKeys = {
  all: ['users'] as const,
  lists: () => [...userKeys.all, 'list'] as const,
  list: (filters: UserFilters) => [...userKeys.lists(), filters] as const,
  details: () => [...userKeys.all, 'detail'] as const,
  detail: (id: string) => [...userKeys.details(), id] as const,
};

// API functions
const userApi = {
  getAll: async (filters: UserFilters): Promise<User[]> => {
    const response = await fetch(`/api/users?${new URLSearchParams(filters)}`);
    if (!response.ok) throw new Error('Failed to fetch users');
    return response.json();
  },
  
  getById: async (id: string): Promise<User> => {
    const response = await fetch(`/api/users/${id}`);
    if (!response.ok) throw new Error('Failed to fetch user');
    return response.json();
  },
  
  create: async (data: CreateUserDTO): Promise<User> => {
    const response = await fetch('/api/users', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify(data),
    });
    if (!response.ok) throw new Error('Failed to create user');
    return response.json();
  },
  
  update: async (id: string, data: Partial<User>): Promise<User> => {
    const response = await fetch(`/api/users/${id}`, {
      method: 'PATCH',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify(data),
    });
    if (!response.ok) throw new Error('Failed to update user');
    return response.json();
  },
  
  delete: async (id: string): Promise<void> => {
    const response = await fetch(`/api/users/${id}`, { method: 'DELETE' });
    if (!response.ok) throw new Error('Failed to delete user');
  },
};

// Hooks
function useUsers(filters: UserFilters) {
  return useQuery({
    queryKey: userKeys.list(filters),
    queryFn: () => userApi.getAll(filters),
    staleTime: 5 * 60 * 1000, // 5 minutes
  });
}

function useUser(id: string) {
  return useQuery({
    queryKey: userKeys.detail(id),
    queryFn: () => userApi.getById(id),
    enabled: !!id,
  });
}

function useCreateUser() {
  const queryClient = useQueryClient();
  
  return useMutation({
    mutationFn: userApi.create,
    onSuccess: (newUser) => {
      // Invalidate and refetch user lists
      queryClient.invalidateQueries({ queryKey: userKeys.lists() });
      
      // Optionally set the new user in cache
      queryClient.setQueryData(userKeys.detail(newUser.id), newUser);
    },
  });
}

function useUpdateUser() {
  const queryClient = useQueryClient();
  
  return useMutation({
    mutationFn: ({ id, data }: { id: string; data: Partial<User> }) =>
      userApi.update(id, data),
    onMutate: async ({ id, data }) => {
      // Cancel outgoing refetches
      await queryClient.cancelQueries({ queryKey: userKeys.detail(id) });
      
      // Snapshot previous value
      const previousUser = queryClient.getQueryData<User>(userKeys.detail(id));
      
      // Optimistically update
      if (previousUser) {
        queryClient.setQueryData(userKeys.detail(id), {
          ...previousUser,
          ...data,
        });
      }
      
      return { previousUser };
    },
    onError: (err, { id }, context) => {
      // Rollback on error
      if (context?.previousUser) {
        queryClient.setQueryData(userKeys.detail(id), context.previousUser);
      }
    },
    onSettled: (_, __, { id }) => {
      // Always refetch after error or success
      queryClient.invalidateQueries({ queryKey: userKeys.detail(id) });
      queryClient.invalidateQueries({ queryKey: userKeys.lists() });
    },
  });
}

// Usage in component
function UserProfile({ userId }: { userId: string }) {
  const { data: user, isLoading, error } = useUser(userId);
  const updateUser = useUpdateUser();
  
  if (isLoading) return <Spinner />;
  if (error) return <ErrorMessage error={error} />;
  if (!user) return <NotFound />;
  
  const handleNameChange = async (newName: string) => {
    await updateUser.mutateAsync({ id: userId, data: { name: newName } });
  };
  
  return (
    <div>
      <h1>{user.name}</h1>
      <p>{user.email}</p>
      <button
        onClick={() => handleNameChange('New Name')}
        disabled={updateUser.isPending}
      >
        {updateUser.isPending ? 'Updating...' : 'Update Name'}
      </button>
    </div>
  );
}
```

---

[Continuing with Java, Go, DevOps, Kafka patterns in same format...]

========================================
SECTION J: JAVA/SPRING BOOT PATTERNS (150+)
========================================

## J.1 CORE JAVA IDIOMS

J.1.1 Records for Immutable Data Transfer
- Use when: Creating DTOs, value objects, or any immutable data carrier
- Context: Use records instead of classes with manual getters/equals/hashCode
```java
// ✅ Good: Record for DTO
public record CreateUserRequest(
    @NotBlank String email,
    @NotBlank @Size(min = 2, max = 100) String name,
    @NotNull Role role
) {
    // Compact constructor for validation/normalization
    public CreateUserRequest {
        email = email.toLowerCase().trim();
        name = name.trim();
        Objects.requireNonNull(email, "email is required");
        Objects.requireNonNull(name, "name is required");
        Objects.requireNonNull(role, "role is required");
    }
    
    // Static factory methods for named construction
    public static CreateUserRequest adminUser(String email, String name) {
        return new CreateUserRequest(email, name, Role.ADMIN);
    }
}

// ✅ Good: Record for value object with domain logic
public record Money(BigDecimal amount, Currency currency) {
    public Money {
        Objects.requireNonNull(amount, "amount must not be null");
        Objects.requireNonNull(currency, "currency must not be null");
        if (amount.compareTo(BigDecimal.ZERO) < 0) {
            throw new IllegalArgumentException("amount must not be negative");
        }
        // Normalize scale
        amount = amount.setScale(2, RoundingMode.HALF_UP);
    }
    
    public static Money zero(Currency currency) {
        return new Money(BigDecimal.ZERO, currency);
    }
    
    public static Money of(double amount, Currency currency) {
        return new Money(BigDecimal.valueOf(amount), currency);
    }
    
    public Money add(Money other) {
        requireSameCurrency(other);
        return new Money(this.amount.add(other.amount), this.currency);
    }
    
    public Money subtract(Money other) {
        requireSameCurrency(other);
        return new Money(this.amount.subtract(other.amount), this.currency);
    }
    
    public Money multiply(int quantity) {
        return new Money(this.amount.multiply(BigDecimal.valueOf(quantity)), this.currency);
    }
    
    private void requireSameCurrency(Money other) {
        if (!this.currency.equals(other.currency)) {
            throw new IllegalArgumentException(
                "Cannot operate on different currencies: %s vs %s"
                    .formatted(this.currency, other.currency)
            );
        }
    }
}

// ✅ Good: Record for query projection
public record UserSummary(
    String id,
    String name,
    String email,
    long orderCount,
    LocalDateTime lastOrderDate
) {}
```
- Avoid/Anti-pattern: Mutable POJOs with setters
```java
// ❌ Bad: Mutable, verbose, error-prone
public class CreateUserRequest {
    private String email;
    private String name;
    private Role role;
    
    public String getEmail() { return email; }
    public void setEmail(String email) { this.email = email; }
    // ... 50+ lines of boilerplate
}
```

J.1.2 Sealed Classes for Domain Modeling
- Use when: Modeling closed hierarchies with exhaustive pattern matching
- Context: Use sealed classes with permits clause for controlled inheritance
```java
// Domain events as sealed hierarchy
public sealed interface OrderEvent permits 
    OrderCreated, OrderConfirmed, OrderShipped, OrderDelivered, OrderCancelled {
    
    String orderId();
    Instant occurredAt();
}

public record OrderCreated(
    String orderId,
    Instant occurredAt,
    String customerId,
    List<OrderItem> items,
    Money total
) implements OrderEvent {}

public record OrderConfirmed(
    String orderId,
    Instant occurredAt,
    String confirmedBy
) implements OrderEvent {}

public record OrderShipped(
    String orderId,
    Instant occurredAt,
    String trackingNumber,
    String carrier
) implements OrderEvent {}

public record OrderDelivered(
    String orderId,
    Instant occurredAt,
    String signedBy
) implements OrderEvent {}

public record OrderCancelled(
    String orderId,
    Instant occurredAt,
    String reason,
    String cancelledBy
) implements OrderEvent {}

// Pattern matching (Java 21+)
public String describeEvent(OrderEvent event) {
    return switch (event) {
        case OrderCreated e -> "Order %s created with %d items, total: %s"
            .formatted(e.orderId(), e.items().size(), e.total());
        case OrderConfirmed e -> "Order %s confirmed by %s"
            .formatted(e.orderId(), e.confirmedBy());
        case OrderShipped e -> "Order %s shipped via %s, tracking: %s"
            .formatted(e.orderId(), e.carrier(), e.trackingNumber());
        case OrderDelivered e -> "Order %s delivered, signed by %s"
            .formatted(e.orderId(), e.signedBy());
        case OrderCancelled e -> "Order %s cancelled: %s"
            .formatted(e.orderId(), e.reason());
        // No default needed - compiler ensures exhaustiveness
    };
}

// Result type with sealed classes
public sealed interface Result<T> permits Result.Success, Result.Failure {
    
    record Success<T>(T value) implements Result<T> {}
    record Failure<T>(String error, String code) implements Result<T> {}
    
    static <T> Result<T> success(T value) {
        return new Success<>(value);
    }
    
    static <T> Result<T> failure(String error, String code) {
        return new Failure<>(error, code);
    }
    
    default T getOrThrow() {
        return switch (this) {
            case Success<T> s -> s.value();
            case Failure<T> f -> throw new RuntimeException(f.error());
        };
    }
    
    default <U> Result<U> map(Function<T, U> mapper) {
        return switch (this) {
            case Success<T> s -> new Success<>(mapper.apply(s.value()));
            case Failure<T> f -> new Failure<>(f.error(), f.code());
        };
    }
    
    default <U> Result<U> flatMap(Function<T, Result<U>> mapper) {
        return switch (this) {
            case Success<T> s -> mapper.apply(s.value());
            case Failure<T> f -> new Failure<>(f.error(), f.code());
        };
    }
}
```

J.1.3 Optional for Nullable Returns
- Use when: Method might not return a value
- Context: Use Optional for return types, NEVER for parameters or fields
```java
public interface UserRepository extends JpaRepository<User, String> {
    
    // ✅ Good: Optional for nullable return
    Optional<User> findByEmail(String email);
    
    // ✅ Good: List for multiple results (empty, not null)
    List<User> findByRole(Role role);
    
    // ✅ Good: Stream for large results
    Stream<User> findAllByCreatedAtAfter(Instant after);
}

// ❌ Bad: Never use Optional for parameters
// void updateEmail(String id, Optional<String> newEmail);

// Using Optional effectively
@Service
@RequiredArgsConstructor
public class UserService {
    private final UserRepository repository;
    
    // orElseThrow for required values
    public UserDTO getUser(String id) {
        return repository.findById(id)
            .map(this::toDTO)
            .orElseThrow(() -> new UserNotFoundException(id));
    }
    
    // orElseGet for default values
    public UserDTO getUserOrAnonymous(String id) {
        return repository.findById(id)
            .map(this::toDTO)
            .orElseGet(UserDTO::anonymous);
    }
    
    // Chaining Optionals
    public Optional<String> getUserEmail(String userId) {
        return repository.findById(userId)
            .map(User::getEmail)
            .filter(email -> !email.isBlank());
    }
    
    // flatMap for nested Optionals
    public Optional<Address> getUserShippingAddress(String userId) {
        return repository.findById(userId)
            .flatMap(User::getShippingAddress); // User::getShippingAddress returns Optional<Address>
    }
    
    // Combining multiple Optionals
    public Optional<OrderSummary> createOrderSummary(String userId, String productId) {
        Optional<User> user = repository.findById(userId);
        Optional<Product> product = productRepository.findById(productId);
        
        return user.flatMap(u -> 
            product.map(p -> new OrderSummary(u, p))
        );
    }
    
    // ifPresentOrElse for conditional actions
    public void notifyUser(String userId, String message) {
        repository.findById(userId).ifPresentOrElse(
            user -> notificationService.send(user.getEmail(), message),
            () -> log.warn("User {} not found, notification skipped", userId)
        );
    }
}
```
- Avoid/Anti-pattern: Optional.get() without checking
```java
// ❌ Bad - defeats the purpose
Optional<User> userOpt = repository.findById(id);
User user = userOpt.get(); // NoSuchElementException if empty

// ❌ Bad - verbose null check
if (userOpt.isPresent()) {
    User user = userOpt.get(); // Don't do this
}

// ✅ Good - use combinators
User user = userOpt.orElseThrow(() -> new UserNotFoundException(id));
```

J.1.4 Streams API Best Practices
- Use when: Processing collections declaratively
- Context: Use streams for transformation, filtering, aggregation
```java
public class StreamPatterns {
    
    // Basic transformations
    public List<UserDTO> toUserDTOs(List<User> users) {
        return users.stream()
            .filter(User::isActive)
            .map(this::toDTO)
            .sorted(Comparator.comparing(UserDTO::name))
            .toList(); // Java 16+ (prefer over collect(Collectors.toList()))
    }
    
    // Grouping
    public Map<Role, List<User>> groupByRole(List<User> users) {
        return users.stream()
            .collect(Collectors.groupingBy(User::getRole));
    }
    
    // Grouping with downstream collector
    public Map<Role, Long> countByRole(List<User> users) {
        return users.stream()
            .collect(Collectors.groupingBy(
                User::getRole,
                Collectors.counting()
            ));
    }
    
    // Grouping with mapping
    public Map<Role, Set<String>> emailsByRole(List<User> users) {
        return users.stream()
            .collect(Collectors.groupingBy(
                User::getRole,
                Collectors.mapping(User::getEmail, Collectors.toSet())
            ));
    }
    
    // Partitioning
    public Map<Boolean, List<User>> partitionByActive(List<User> users) {
        return users.stream()
            .collect(Collectors.partitioningBy(User::isActive));
    }
    
    // FlatMap for nested collections
    public List<OrderItem> getAllOrderItems(List<Order> orders) {
        return orders.stream()
            .flatMap(order -> order.getItems().stream())
            .toList();
    }
    
    // Reduce for aggregation
    public Money calculateTotal(List<OrderItem> items) {
        return items.stream()
            .map(item -> item.getPrice().multiply(item.getQuantity()))
            .reduce(Money.zero(Currency.USD), Money::add);
    }
    
    // Max/Min with Optional
    public Optional<User> findHighestSpender(List<User> users) {
        return users.stream()
            .max(Comparator.comparing(User::getTotalSpent));
    }
    
    // Collectors.teeing (Java 12+)
    public record Statistics(long count, double average) {}
    
    public Statistics calculateStatistics(List<Order> orders) {
        return orders.stream()
            .map(Order::getTotal)
            .collect(Collectors.teeing(
                Collectors.counting(),
                Collectors.averagingDouble(BigDecimal::doubleValue),
                Statistics::new
            ));
    }
    
    // toMap with merge function
    public Map<String, User> latestUserByEmail(List<User> users) {
        return users.stream()
            .collect(Collectors.toMap(
                User::getEmail,
                Function.identity(),
                (existing, replacement) -> 
                    existing.getCreatedAt().isAfter(replacement.getCreatedAt()) 
                        ? existing : replacement
            ));
    }
}
```
- Avoid/Anti-pattern: Side effects in streams
```java
// ❌ Bad: Side effects
List<User> activeUsers = new ArrayList<>();
users.stream()
    .filter(User::isActive)
    .forEach(activeUsers::add); // Side effect!

// ✅ Good: Pure transformation
List<User> activeUsers = users.stream()
    .filter(User::isActive)
    .toList();
```

J.1.5 Virtual Threads (Java 21+)
- Use when: High-throughput I/O-bound applications
- Context: Use virtual threads for blocking operations
```java
// Enable virtual threads in Spring Boot 3.2+
// application.yml:
// spring:
//   threads:
//     virtual:
//       enabled: true

// Manual virtual thread usage
public class VirtualThreadExample {
    
    public List<UserDTO> fetchAllUsersParallel(List<String> userIds) {
        try (var executor = Executors.newVirtualThreadPerTaskExecutor()) {
            List<Future<UserDTO>> futures = userIds.stream()
                .map(id -> executor.submit(() -> fetchUser(id)))
                .toList();
            
            return futures.stream()
                .map(this::getFutureResult)
                .filter(Objects::nonNull)
                .toList();
        }
    }
    
    private UserDTO getFutureResult(Future<UserDTO> future) {
        try {
            return future.get(5, TimeUnit.SECONDS);
        } catch (Exception e) {
            log.error("Failed to fetch user", e);
            return null;
        }
    }
    
    // Structured concurrency (Java 21+ Preview)
    public UserProfile fetchUserProfile(String userId) throws Exception {
        try (var scope = new StructuredTaskScope.ShutdownOnFailure()) {
            Subtask<User> userTask = scope.fork(() -> 
                userRepository.findById(userId).orElseThrow()
            );
            Subtask<List<Order>> ordersTask = scope.fork(() -> 
                orderRepository.findByUserId(userId)
            );
            Subtask<Double> ratingTask = scope.fork(() -> 
                reviewService.getAverageRating(userId)
            );
            
            scope.join();
            scope.throwIfFailed();
            
            return new UserProfile(
                userTask.get(),
                ordersTask.get(),
                ratingTask.get()
            );
        }
    }
}

// Thread-safe service pattern with virtual threads
@Service
public class UserService {
    // Thread-safe: final fields, immutable cache config
    private final UserRepository repository;
    private final LoadingCache<String, UserDTO> cache;
    
    public UserService(UserRepository repository) {
        this.repository = repository;
        this.cache = Caffeine.newBuilder()
            .maximumSize(10_000)
            .expireAfterWrite(Duration.ofMinutes(5))
            .build(this::loadUser);
    }
    
    public UserDTO getUser(String id) {
        return cache.get(id); // Thread-safe
    }
    
    private UserDTO loadUser(String id) {
        return repository.findById(id)
            .map(this::toDTO)
            .orElseThrow(() -> new UserNotFoundException(id));
    }
}
```

---

[Continuing with more Java patterns, then Go, DevOps, Kafka...]

========================================
SECTION G: GOLANG PATTERNS (150+)
========================================

## G.1 ERROR HANDLING

G.1.1 Error Wrapping with Context
- Use when: Propagating errors with additional context
- Context: Use fmt.Errorf with %w verb or errors.Join
```go
import (
    "errors"
    "fmt"
)

// Define sentinel errors
var (
    ErrNotFound      = errors.New("not found")
    ErrUnauthorized  = errors.New("unauthorized")
    ErrInvalidInput  = errors.New("invalid input")
    ErrConflict      = errors.New("conflict")
)

// Custom error type with structured data
type ValidationError struct {
    Field   string
    Message string
}

func (e *ValidationError) Error() string {
    return fmt.Sprintf("validation error on field %q: %s", e.Field, e.Message)
}

// Error wrapping pattern
func (s *UserService) GetUser(ctx context.Context, id string) (*User, error) {
    user, err := s.repo.FindByID(ctx, id)
    if err != nil {
        if errors.Is(err, sql.ErrNoRows) {
            return nil, fmt.Errorf("user %s: %w", id, ErrNotFound)
        }
        return nil, fmt.Errorf("fetching user %s: %w", id, err)
    }
    return user, nil
}

// Error checking in handlers
func (h *Handler) GetUser(w http.ResponseWriter, r *http.Request) {
    ctx := r.Context()
    id := r.PathValue("id")
    
    user, err := h.userService.GetUser(ctx, id)
    if err != nil {
        switch {
        case errors.Is(err, ErrNotFound):
            h.respondError(w, http.StatusNotFound, "User not found")
        case errors.Is(err, ErrUnauthorized):
            h.respondError(w, http.StatusUnauthorized, "Unauthorized")
        default:
            h.logger.Error("failed to get user", "error", err, "userId", id)
            h.respondError(w, http.StatusInternalServerError, "Internal error")
        }
        return
    }
    
    h.respondJSON(w, http.StatusOK, user)
}

// Extracting typed errors
func handleError(err error) {
    var validationErr *ValidationError
    if errors.As(err, &validationErr) {
        fmt.Printf("Validation failed on %s: %s\n", validationErr.Field, validationErr.Message)
        return
    }
    
    // Check sentinel errors
    if errors.Is(err, ErrNotFound) {
        fmt.Println("Resource not found")
        return
    }
    
    fmt.Printf("Unknown error: %v\n", err)
}

// Multiple error aggregation (Go 1.20+)
func validateUser(u *User) error {
    var errs []error
    
    if u.Email == "" {
        errs = append(errs, &ValidationError{Field: "email", Message: "required"})
    }
    if u.Name == "" {
        errs = append(errs, &ValidationError{Field: "name", Message: "required"})
    }
    if len(u.Password) < 8 {
        errs = append(errs, &ValidationError{Field: "password", Message: "must be at least 8 characters"})
    }
    
    if len(errs) > 0 {
        return errors.Join(errs...)
    }
    return nil
}
```
- Avoid/Anti-pattern: Ignoring errors or using panic
```go
// ❌ Bad: Ignoring error
user, _ := userService.GetUser(ctx, id)

// ❌ Bad: Using panic for recoverable errors
func GetUser(ctx context.Context, id string) *User {
    user, err := repo.FindByID(ctx, id)
    if err != nil {
        panic(err) // Don't do this for recoverable errors!
    }
    return user
}

// ❌ Bad: No context in error
if err != nil {
    return err // Loses context of where error occurred
}
```

G.1.2 Table-Driven Tests
- Use when: Testing functions with multiple input/output scenarios
- Context: Use test tables with descriptive names and subtests
```go
func TestCalculateDiscount(t *testing.T) {
    tests := []struct {
        name     string
        amount   float64
        userType string
        want     float64
        wantErr  error
    }{
        {
            name:     "regular user no discount",
            amount:   100.0,
            userType: "regular",
            want:     100.0,
            wantErr:  nil,
        },
        {
            name:     "premium user 10% discount",
            amount:   100.0,
            userType: "premium",
            want:     90.0,
            wantErr:  nil,
        },
        {
            name:     "vip user 20% discount",
            amount:   100.0,
            userType: "vip",
            want:     80.0,
            wantErr:  nil,
        },
        {
            name:     "negative amount returns error",
            amount:   -50.0,
            userType: "regular",
            want:     0,
            wantErr:  ErrInvalidAmount,
        },
        {
            name:     "unknown user type returns error",
            amount:   100.0,
            userType: "unknown",
            want:     0,
            wantErr:  ErrUnknownUserType,
        },
        {
            name:     "zero amount allowed",
            amount:   0,
            userType: "regular",
            want:     0,
            wantErr:  nil,
        },
    }

    for _, tt := range tests {
        t.Run(tt.name, func(t *testing.T) {
            got, err := CalculateDiscount(tt.amount, tt.userType)

            if !errors.Is(err, tt.wantErr) {
                t.Errorf("CalculateDiscount() error = %v, wantErr %v", err, tt.wantErr)
                return
            }

            if got != tt.want {
                t.Errorf("CalculateDiscount() = %v, want %v", got, tt.want)
            }
        })
    }
}

// With testify for cleaner assertions
func TestUserService_CreateUser(t *testing.T) {
    tests := []struct {
        name      string
        setupMock func(*MockUserRepository)
        input     CreateUserRequest
        want      *User
        wantErr   string
    }{
        {
            name: "success",
            setupMock: func(m *MockUserRepository) {
                m.EXPECT().
                    ExistsByEmail(gomock.Any(), "test@example.com").
                    Return(false, nil)
                m.EXPECT().
                    Save(gomock.Any(), gomock.Any()).
                    Return(nil)
            },
            input:   CreateUserRequest{Email: "test@example.com", Name: "Test"},
            want:    &User{Email: "test@example.com", Name: "Test"},
            wantErr: "",
        },
        {
            name: "duplicate email",
            setupMock: func(m *MockUserRepository) {
                m.EXPECT().
                    ExistsByEmail(gomock.Any(), "existing@example.com").
                    Return(true, nil)
            },
            input:   CreateUserRequest{Email: "existing@example.com", Name: "Test"},
            want:    nil,
            wantErr: "email already exists",
        },
    }

    for _, tt := range tests {
        t.Run(tt.name, func(t *testing.T) {
            ctrl := gomock.NewController(t)
            defer ctrl.Finish()

            mockRepo := NewMockUserRepository(ctrl)
            tt.setupMock(mockRepo)

            service := NewUserService(mockRepo)
            got, err := service.CreateUser(context.Background(), tt.input)

            if tt.wantErr != "" {
                require.Error(t, err)
                assert.Contains(t, err.Error(), tt.wantErr)
                return
            }

            require.NoError(t, err)
            assert.Equal(t, tt.want.Email, got.Email)
            assert.Equal(t, tt.want.Name, got.Name)
        })
    }
}
```

G.1.3 Interface Design - Accept Interfaces, Return Structs
- Use when: Defining contracts between components
- Context: Keep interfaces small and focused
```go
// Small, focused interfaces
type Reader interface {
    Read(p []byte) (n int, err error)
}

type Writer interface {
    Write(p []byte) (n int, err error)
}

type ReadWriter interface {
    Reader
    Writer
}

// Repository interface - accept interface
type UserRepository interface {
    FindByID(ctx context.Context, id string) (*User, error)
    FindByEmail(ctx context.Context, email string) (*User, error)
    Save(ctx context.Context, user *User) error
    Delete(ctx context.Context, id string) error
}

// Service accepts interfaces, returns concrete types
type UserService struct {
    repo   UserRepository  // Interface
    events EventPublisher  // Interface
    logger *slog.Logger    // Concrete (usually fine for loggers)
}

func NewUserService(repo UserRepository, events EventPublisher, logger *slog.Logger) *UserService {
    return &UserService{
        repo:   repo,
        events: events,
        logger: logger,
    }
}

// Concrete implementation
type PostgresUserRepository struct {
    db *sql.DB
}

func NewPostgresUserRepository(db *sql.DB) *PostgresUserRepository {
    return &PostgresUserRepository{db: db}
}

func (r *PostgresUserRepository) FindByID(ctx context.Context, id string) (*User, error) {
    var user User
    err := r.db.QueryRowContext(ctx,
        "SELECT id, email, name, created_at FROM users WHERE id = $1",
        id,
    ).Scan(&user.ID, &user.Email, &user.Name, &user.CreatedAt)
    
    if err != nil {
        if errors.Is(err, sql.ErrNoRows) {
            return nil, fmt.Errorf("user %s: %w", id, ErrNotFound)
        }
        return nil, fmt.Errorf("querying user %s: %w", id, err)
    }
    
    return &user, nil
}

// Functional options pattern
type Option func(*Config)

func WithTimeout(d time.Duration) Option {
    return func(c *Config) {
        c.timeout = d
    }
}

func WithRetries(n int) Option {
    return func(c *Config) {
        c.retries = n
    }
}

func WithLogger(logger *slog.Logger) Option {
    return func(c *Config) {
        c.logger = logger
    }
}

func NewClient(baseURL string, opts ...Option) *Client {
    cfg := &Config{
        timeout: 30 * time.Second,
        retries: 3,
        logger:  slog.Default(),
    }
    
    for _, opt := range opts {
        opt(cfg)
    }
    
    return &Client{
        baseURL: baseURL,
        config:  cfg,
        http:    &http.Client{Timeout: cfg.timeout},
    }
}

// Usage
client := NewClient("https://api.example.com",
    WithTimeout(10*time.Second),
    WithRetries(5),
)
```

---

========================================
SECTION K: APACHE KAFKA PATTERNS (100+)
========================================

## K.1 PRODUCER PATTERNS

K.1.1 Idempotent Producer Configuration
- Use when: Ensuring exactly-once delivery semantics
- Context: Enable idempotence with proper retries and acks
```java
@Configuration
public class KafkaProducerConfig {
    
    @Value("${spring.kafka.bootstrap-servers}")
    private String bootstrapServers;
    
    @Bean
    public ProducerFactory<String, Object> producerFactory() {
        Map<String, Object> config = new HashMap<>();
        
        // Basic configuration
        config.put(ProducerConfig.BOOTSTRAP_SERVERS_CONFIG, bootstrapServers);
        config.put(ProducerConfig.KEY_SERIALIZER_CLASS_CONFIG, StringSerializer.class);
        config.put(ProducerConfig.VALUE_SERIALIZER_CLASS_CONFIG, JsonSerializer.class);
        
        // Idempotence settings - CRITICAL
        config.put(ProducerConfig.ENABLE_IDEMPOTENCE_CONFIG, true);
        config.put(ProducerConfig.ACKS_CONFIG, "all");
        config.put(ProducerConfig.RETRIES_CONFIG, Integer.MAX_VALUE);
        config.put(ProducerConfig.MAX_IN_FLIGHT_REQUESTS_PER_CONNECTION, 5);
        
        // Delivery timeout
        config.put(ProducerConfig.DELIVERY_TIMEOUT_MS_CONFIG, 120000);
        config.put(ProducerConfig.REQUEST_TIMEOUT_MS_CONFIG, 30000);
        
        // Batching for throughput
        config.put(ProducerConfig.BATCH_SIZE_CONFIG, 16384);
        config.put(ProducerConfig.LINGER_MS_CONFIG, 10);
        config.put(ProducerConfig.COMPRESSION_TYPE_CONFIG, "snappy");
        
        // Memory management
        config.put(ProducerConfig.BUFFER_MEMORY_CONFIG, 33554432); // 32MB
        
        return new DefaultKafkaProducerFactory<>(config);
    }
    
    @Bean
    public KafkaTemplate<String, Object> kafkaTemplate() {
        KafkaTemplate<String, Object> template = new KafkaTemplate<>(producerFactory());
        template.setObservationEnabled(true); // Enable metrics
        return template;
    }
}

// Producer service with proper error handling
@Service
@Slf4j
@RequiredArgsConstructor
public class OrderEventProducer {
    
    private final KafkaTemplate<String, Object> kafkaTemplate;
    
    public CompletableFuture<SendResult<String, Object>> publishOrderCreated(
            OrderCreatedEvent event) {
        
        return kafkaTemplate.send("orders.created.v1", event.orderId(), event)
            .whenComplete((result, ex) -> {
                if (ex != null) {
                    log.error("Failed to publish OrderCreated event for order {}", 
                        event.orderId(), ex);
                    // Could trigger alerting, dead letter queue, etc.
                } else {
                    log.info("OrderCreated event published: topic={}, partition={}, offset={}, orderId={}",
                        result.getRecordMetadata().topic(),
                        result.getRecordMetadata().partition(),
                        result.getRecordMetadata().offset(),
                        event.orderId());
                }
            });
    }
    
    // Synchronous send when you need to know result immediately
    public void publishOrderCreatedSync(OrderCreatedEvent event) throws Exception {
        try {
            SendResult<String, Object> result = kafkaTemplate
                .send("orders.created.v1", event.orderId(), event)
                .get(10, TimeUnit.SECONDS);
            
            log.info("Event published synchronously: offset={}", 
                result.getRecordMetadata().offset());
        } catch (ExecutionException e) {
            log.error("Failed to publish event", e.getCause());
            throw new EventPublishException("Failed to publish order event", e.getCause());
        } catch (TimeoutException e) {
            log.error("Timeout publishing event", e);
            throw new EventPublishException("Timeout publishing order event", e);
        }
    }
    
    // Batch publishing
    public void publishOrdersBatch(List<OrderCreatedEvent> events) {
        List<CompletableFuture<SendResult<String, Object>>> futures = events.stream()
            .map(event -> kafkaTemplate.send("orders.created.v1", event.orderId(), event))
            .toList();
        
        CompletableFuture.allOf(futures.toArray(new CompletableFuture[0]))
            .whenComplete((result, ex) -> {
                if (ex != null) {
                    log.error("Some events failed to publish", ex);
                } else {
                    log.info("Batch of {} events published successfully", events.size());
                }
            });
    }
}
```
- Avoid/Anti-pattern: Fire and forget without error handling
```java
// ❌ Bad: No error handling, no confirmation
kafkaTemplate.send("orders.created.v1", event.orderId(), event);
// What if it fails? You'll never know!
```

K.1.2 Consumer with Manual Offset Commit
- Use when: Controlling offset commits for at-least-once processing
- Context: Disable auto-commit, commit after successful processing
```java
@Configuration
public class KafkaConsumerConfig {
    
    @Bean
    public ConsumerFactory<String, OrderCreatedEvent> consumerFactory() {
        Map<String, Object> config = new HashMap<>();
        
        config.put(ConsumerConfig.BOOTSTRAP_SERVERS_CONFIG, bootstrapServers);
        config.put(ConsumerConfig.GROUP_ID_CONFIG, "order-processor-group");
        config.put(ConsumerConfig.KEY_DESERIALIZER_CLASS_CONFIG, StringDeserializer.class);
        config.put(ConsumerConfig.VALUE_DESERIALIZER_CLASS_CONFIG, JsonDeserializer.class);
        
        // Manual commit - CRITICAL for at-least-once
        config.put(ConsumerConfig.ENABLE_AUTO_COMMIT_CONFIG, false);
        
        // Consumer tuning
        config.put(ConsumerConfig.MAX_POLL_RECORDS_CONFIG, 100);
        config.put(ConsumerConfig.MAX_POLL_INTERVAL_MS_CONFIG, 300000); // 5 minutes
        config.put(ConsumerConfig.SESSION_TIMEOUT_MS_CONFIG, 45000);
        config.put(ConsumerConfig.HEARTBEAT_INTERVAL_MS_CONFIG, 15000);
        
        // Start from earliest if no committed offset
        config.put(ConsumerConfig.AUTO_OFFSET_RESET_CONFIG, "earliest");
        
        // Deserializer config
        config.put(JsonDeserializer.TRUSTED_PACKAGES, "com.example.events");
        config.put(JsonDeserializer.USE_TYPE_INFO_HEADERS, false);
        config.put(JsonDeserializer.VALUE_DEFAULT_TYPE, OrderCreatedEvent.class);
        
        return new DefaultKafkaConsumerFactory<>(config);
    }
    
    @Bean
    public ConcurrentKafkaListenerContainerFactory<String, OrderCreatedEvent> 
            kafkaListenerContainerFactory() {
        
        var factory = new ConcurrentKafkaListenerContainerFactory<String, OrderCreatedEvent>();
        factory.setConsumerFactory(consumerFactory());
        factory.setConcurrency(3); // Number of consumer threads
        factory.getContainerProperties().setAckMode(AckMode.MANUAL_IMMEDIATE);
        
        // Error handling with dead letter topic
        factory.setCommonErrorHandler(new DefaultErrorHandler(
            new DeadLetterPublishingRecoverer(kafkaTemplate,
                (record, ex) -> new TopicPartition(
                    record.topic() + ".DLT", 
                    record.partition()
                )),
            new FixedBackOff(1000L, 3L) // 1 second, 3 retries
        ));
        
        return factory;
    }
}

@Service
@Slf4j
@RequiredArgsConstructor
public class OrderEventConsumer {
    
    private final OrderService orderService;
    private final IdempotencyService idempotencyService;
    
    @KafkaListener(
        topics = "orders.created.v1",
        groupId = "order-processor-group",
        containerFactory = "kafkaListenerContainerFactory"
    )
    public void handleOrderCreated(
            @Payload OrderCreatedEvent event,
            @Header(KafkaHeaders.RECEIVED_KEY) String key,
            @Header(KafkaHeaders.RECEIVED_PARTITION) int partition,
            @Header(KafkaHeaders.OFFSET) long offset,
            @Header(KafkaHeaders.RECEIVED_TIMESTAMP) long timestamp,
            Acknowledgment acknowledgment) {
        
        // Create idempotency key from topic/partition/offset
        String idempotencyKey = String.format("orders.created.v1-%d-%d", partition, offset);
        
        try {
            // Idempotency check - prevent duplicate processing
            if (idempotencyService.isDuplicate(idempotencyKey)) {
                log.info("Duplicate event detected, skipping: key={}, partition={}, offset={}", 
                    key, partition, offset);
                acknowledgment.acknowledge();
                return;
            }
            
            // Process the event
            log.info("Processing OrderCreated: orderId={}, partition={}, offset={}", 
                event.orderId(), partition, offset);
            
            orderService.processOrderCreated(event);
            
            // Mark as processed BEFORE acknowledging
            idempotencyService.markProcessed(idempotencyKey, Duration.ofDays(7));
            
            // Commit offset AFTER successful processing
            acknowledgment.acknowledge();
            
            log.info("Successfully processed OrderCreated: orderId={}", event.orderId());
            
        } catch (RetryableException e) {
            // Don't acknowledge - will be redelivered
            log.warn("Retryable error processing order {}, will retry: {}", 
                event.orderId(), e.getMessage());
            throw e;
            
        } catch (Exception e) {
            // Non-retryable error - acknowledge to prevent infinite loop
            // Error handler will send to DLT
            log.error("Non-retryable error processing order {}: {}", 
                event.orderId(), e.getMessage(), e);
            throw e;
        }
    }
    
    // Batch listener for high throughput
    @KafkaListener(
        topics = "orders.events.v1",
        groupId = "order-batch-processor",
        containerFactory = "batchKafkaListenerContainerFactory"
    )
    public void handleOrderEventsBatch(
            List<OrderEvent> events,
            Acknowledgment acknowledgment) {
        
        log.info("Received batch of {} events", events.size());
        
        try {
            orderService.processEventsBatch(events);
            acknowledgment.acknowledge();
        } catch (Exception e) {
            log.error("Failed to process batch", e);
            throw e;
        }
    }
}
```

---

========================================
SECTION D: DEVOPS PATTERNS (100+)
========================================

## D.1 TERRAFORM PATTERNS

D.1.1 Module Design with Inputs/Outputs
- Use when: Creating reusable infrastructure components
- Context: Define modules with clear interfaces and documentation
```hcl
# modules/aws-eks-cluster/variables.tf
variable "cluster_name" {
  type        = string
  description = "Name of the EKS cluster"
  
  validation {
    condition     = length(var.cluster_name) <= 40
    error_message = "Cluster name must be 40 characters or less."
  }
}

variable "kubernetes_version" {
  type        = string
  description = "Kubernetes version for the cluster"
  default     = "1.29"
  
  validation {
    condition     = can(regex("^1\\.(2[7-9]|3[0-9])$", var.kubernetes_version))
    error_message = "Kubernetes version must be 1.27 or higher."
  }
}

variable "vpc_id" {
  type        = string
  description = "VPC ID where cluster will be deployed"
}

variable "subnet_ids" {
  type        = list(string)
  description = "List of subnet IDs for the cluster"
  
  validation {
    condition     = length(var.subnet_ids) >= 2
    error_message = "At least 2 subnets required for HA."
  }
}

variable "node_groups" {
  type = map(object({
    instance_types = list(string)
    min_size       = number
    max_size       = number
    desired_size   = number
    disk_size      = number
    labels         = map(string)
    taints = list(object({
      key    = string
      value  = string
      effect = string
    }))
  }))
  description = "Map of node group configurations"
  default     = {}
}

variable "tags" {
  type        = map(string)
  description = "Tags to apply to all resources"
  default     = {}
}

# modules/aws-eks-cluster/main.tf
terraform {
  required_version = ">= 1.5"
  required_providers {
    aws = {
      source  = "hashicorp/aws"
      version = "~> 5.0"
    }
  }
}

locals {
  common_tags = merge(var.tags, {
    ManagedBy   = "terraform"
    Module      = "aws-eks-cluster"
    ClusterName = var.cluster_name
  })
}

resource "aws_eks_cluster" "main" {
  name     = var.cluster_name
  version  = var.kubernetes_version
  role_arn = aws_iam_role.cluster.arn
  
  vpc_config {
    subnet_ids              = var.subnet_ids
    endpoint_private_access = true
    endpoint_public_access  = true
    security_group_ids      = [aws_security_group.cluster.id]
  }
  
  enabled_cluster_log_types = ["api", "audit", "authenticator"]
  
  encryption_config {
    provider {
      key_arn = aws_kms_key.eks.arn
    }
    resources = ["secrets"]
  }
  
  tags = local.common_tags
  
  depends_on = [
    aws_iam_role_policy_attachment.cluster_policy,
    aws_cloudwatch_log_group.cluster,
  ]
}

resource "aws_eks_node_group" "main" {
  for_each = var.node_groups
  
  cluster_name    = aws_eks_cluster.main.name
  node_group_name = each.key
  node_role_arn   = aws_iam_role.node.arn
  subnet_ids      = var.subnet_ids
  instance_types  = each.value.instance_types
  disk_size       = each.value.disk_size
  
  scaling_config {
    min_size     = each.value.min_size
    max_size     = each.value.max_size
    desired_size = each.value.desired_size
  }
  
  labels = each.value.labels
  
  dynamic "taint" {
    for_each = each.value.taints
    content {
      key    = taint.value.key
      value  = taint.value.value
      effect = taint.value.effect
    }
  }
  
  tags = merge(local.common_tags, {
    NodeGroup = each.key
  })
  
  depends_on = [
    aws_iam_role_policy_attachment.node_policy,
    aws_iam_role_policy_attachment.cni_policy,
    aws_iam_role_policy_attachment.ecr_policy,
  ]
  
  lifecycle {
    ignore_changes = [scaling_config[0].desired_size]
  }
}

# modules/aws-eks-cluster/outputs.tf
output "cluster_id" {
  description = "EKS cluster ID"
  value       = aws_eks_cluster.main.id
}

output "cluster_endpoint" {
  description = "EKS cluster endpoint"
  value       = aws_eks_cluster.main.endpoint
}

output "cluster_certificate_authority_data" {
  description = "Base64 encoded certificate data"
  value       = aws_eks_cluster.main.certificate_authority[0].data
  sensitive   = true
}

output "cluster_security_group_id" {
  description = "Security group ID attached to the cluster"
  value       = aws_security_group.cluster.id
}

output "oidc_provider_arn" {
  description = "OIDC provider ARN for IRSA"
  value       = aws_iam_openid_connect_provider.eks.arn
}
```

---

## QUALITY CHECKLISTS

### ✅ TypeScript Code Quality
- [ ] `strict: true` enabled in tsconfig
- [ ] No `any` types (use `unknown` for uncertain types)
- [ ] Branded types for domain IDs
- [ ] Zod schemas for external data validation
- [ ] Discriminated unions for state machines
- [ ] Proper error handling (no unhandled rejections)
- [ ] ESLint + Prettier configured and passing

### ✅ Java/Spring Boot Quality
- [ ] Records for DTOs and value objects
- [ ] Optional for nullable returns only (never params/fields)
- [ ] Constructor injection (no field injection)
- [ ] JPA entities with proper equals/hashCode
- [ ] Sealed classes for closed hierarchies
- [ ] Virtual threads for I/O-bound operations (Java 21+)
- [ ] TestContainers for integration tests

### ✅ Go Code Quality
- [ ] Errors wrapped with context (`fmt.Errorf %w`)
- [ ] Context propagated through call chain
- [ ] Interfaces accepted, structs returned
- [ ] Table-driven tests
- [ ] Goroutines properly managed (no leaks)
- [ ] `golangci-lint` clean
- [ ] `go vet` passing

### ✅ Kafka Quality
- [ ] Idempotent producers enabled
- [ ] Manual offset commit after processing
- [ ] Dead letter topics configured
- [ ] Schema Registry for schema evolution
- [ ] Consumer group tuning (poll interval, session timeout)
- [ ] Monitoring (consumer lag, partition distribution)
- [ ] Idempotent consumers (deduplication)

### ✅ DevOps Quality
- [ ] Infrastructure as Code (Terraform/Pulumi)
- [ ] Multi-stage Docker builds
- [ ] Kubernetes resource limits and health checks
- [ ] CI/CD with proper caching
- [ ] Secrets management (not in code)
- [ ] Observability (metrics, logs, traces)
- [ ] GitOps workflow (ArgoCD/Flux)

---

## TOOL RECOMMENDATIONS (2024-2025)

| Category | Primary Choice | Alternative |
|----------|----------------|-------------|
| **TS Testing** | Vitest | Jest |
| **TS Validation** | Zod | Yup, Valibot |
| **TS State** | Zustand, TanStack Query | Redux Toolkit |
| **Java Build** | Gradle (Kotlin DSL) | Maven |
| **Java Testing** | JUnit 5 + AssertJ + Testcontainers | - |
| **Go Testing** | testify + gomock | - |
| **Go Linting** | golangci-lint | - |
| **IaC** | Terraform | Pulumi, CDK |
| **K8s Deploy** | Helm + Kustomize + ArgoCD | Flux |
| **CI/CD** | GitHub Actions | GitLab CI |
| **Kafka Testing** | Testcontainers | EmbeddedKafka |
| **Kafka Schema** | Avro + Schema Registry | Protobuf |
| **Observability** | OpenTelemetry + Grafana | Datadog |


