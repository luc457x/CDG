# When to Mock

Mock at **system boundaries** only:

- External APIs (payment, email, etc.)
- Databases (sometimes — prefer test DB)
- Time/randomness
- File system (sometimes)

Don't mock:

- Own classes/modules
- Internal collaborators
- Anything you control

## Designing for Mockability

At system boundaries, design interfaces easy to mock:

### 1. Use Dependency Injection

Pass external dependencies in rather than creating them internally:

```typescript
// Easy to mock
function processPayment(order, paymentClient) {
  return paymentClient.charge(order.total);
}

// Hard to mock
function processPayment(order) {
  const client = new StripeClient(process.env.STRIPE_KEY);
  return client.charge(order.total);
}
```

### 2. Prefer SDK-Style Interfaces over Generic Fetchers

Create specific functions for each external operation instead of one generic function with conditional logic:

```typescript
// GOOD: Each function independently mockable
const api = {
  getUser: (id) => fetch(`/users/${id}`),
  getOrders: (userId) => fetch(`/users/${userId}/orders`),
  createOrder: (data) => fetch('/orders', { method: 'POST', body: data }),
};

// BAD: Mocking requires conditional logic inside mock
const api = {
  fetch: (endpoint, options) => fetch(endpoint, options),
};
```

SDK approach:

- Each mock returns one specific shape
- No conditional logic in test setup
- Easier to see which endpoints test exercises
- Type safety per endpoint
