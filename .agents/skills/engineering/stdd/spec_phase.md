# Spec Phase — Defining Contracts

## What to Define

Three layers, in order:

### 1. Data Models / Schemas

Core domain types. Nouns of system.

```typescript
// TypeScript example
interface Order {
  id: string;
  userId: string;
  items: OrderItem[];
  status: "pending" | "confirmed" | "shipped" | "cancelled";
  createdAt: Date;
}

interface OrderItem {
  productId: string;
  quantity: number;  // must be > 0
  unitPrice: number; // in cents
}
```

```python
# Python example
@dataclass
class Order:
    id: str
    user_id: str
    items: list[OrderItem]
    status: Literal["pending", "confirmed", "shipped", "cancelled"]
    created_at: datetime
```

Rules:

- Use project's domain glossary (ubiquitous language)
- Encode constraints in types where possible (unions > strings, branded types > primitives)
- Keep models flat — avoid deep nesting

### 2. Public Interfaces

Function signatures, API endpoints, class methods. Verbs.

```typescript
// Repository interface
interface OrderRepository {
  create(data: CreateOrderInput): Promise<Order>;
  findById(id: string): Promise<Order | null>;
  listByUser(userId: string): Promise<Order[]>;
  updateStatus(id: string, status: Order["status"]): Promise<Order>;
}

// API endpoint contract (DTOs)
interface CreateOrderRequest {
  items: { productId: string; quantity: number }[];
}

interface CreateOrderResponse {
  order: Order;
}
```

Rules:

- Define inputs and outputs explicitly (DTOs for APIs)
- Define error cases as part of contract (error types, status codes)
- Accept dependencies, don't create them (testability)
- Keep interfaces small — [deep modules](./deep_modules.md)

### 3. Validation Rules & Constraints

Business rules from SPEC.md encoded as type constraints or validator contracts.

```typescript
// Encode in types when possible
type Quantity = number & { __brand: "positive" }; // branded type

// Encode as validation contract when not
interface OrderValidation {
  maxItemsPerOrder: 50;
  minQuantityPerItem: 1;
  maxQuantityPerItem: 999;
  allowedStatuses: Order["status"][];
}
```

## What NOT to Define

- Internal helper functions
- Private methods
- Database query structure
- Algorithm choice
- File/module organization

These are implementation details. Let TDD drive them.

## Output Criteria

Spec phase done when:

1. All types/schemas compile or parse without errors
2. All public interfaces have defined inputs, outputs, and error cases
3. Business rules from SPEC.md represented in types or validation contracts
4. User reviewed and approved
