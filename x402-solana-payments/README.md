# X402 Solana Payments

A Next.js application that demonstrates paid access to routes using the [X402 payment protocol](https://github.com/coinbase/x402) with Solana settlement. The `x402-next` middleware returns HTTP 402 for unpaid requests, presents a payment flow, verifies the transaction through a facilitator, and issues a session token for subsequent access.

The application includes an AI fortune-teller endpoint backed by OpenAI and two tiered content routes.

> Part of the [Solana Core Topics](../README.md) reference.

## Core Concepts

- HTTP 402 payment negotiation for protected routes
- `x402-next` middleware for route-level pricing
- Solana payment verification through a facilitator
- Session tokens for repeat access after payment
- Next.js App Router middleware and environment-driven configuration

## Payment-Protected Routes

| Route | Price | Content |
| --- | --- | --- |
| `/pay-fortune` | `$0.01` | Payment confirmation page that links to the AI fortune teller. |
| `/content/cheap` | `$0.01` | Budget content page. |
| `/content/expensive` | `$0.25` | Premium content page. |

These are the only paths listed in the middleware matcher. Routes such as `/`, `/fortune`, and `/api/devine` are not payment-protected.

## Architecture

| Component | Responsibility |
| --- | --- |
| `middleware.ts` | Configures `paymentMiddleware` from `x402-next` with the receiver address, network, facilitator, prices, and CDP client key. |
| `app/page.tsx` | Landing page linking to the three protected routes. |
| `app/pay-fortune/page.tsx` | Payment success page; links to `/fortune`. |
| `app/fortune/page.tsx` | Collects birth information and calls `/api/devine`. |
| `app/api/devine/route.ts` | Calls the OpenAI Chat Completions API with `OPENAI_API_KEY`. |
| `app/content/[type]/page.tsx` | Renders the `cheap` or `expensive` content page. |
| `components/cats-component.tsx` | Displays a randomly selected GIF for the content tier. |

### Payment Flow

```
Client                    Next.js middleware                 Facilitator / Solana
  |                              |                                   |
  |-- GET /content/cheap ------->|                                   |
  |<----- 402 Payment Required --|                                   |
  |-- payment via wallet ------->|                                   |
  |                              |-- verify transaction ------------>|
  |                              |<----- verification result --------|
  |<----- session token ---------|                                   |
  |-- GET /content/cheap ------->|                                   |
  |<----- protected content -----|                                   |
```

The middleware handles the 402 response, payment requirements, verification, and session creation. Application route handlers do not contain payment logic.

## Project Structure

```
x402-solana-payments/
├── app/
│   ├── api/devine/route.ts
│   ├── content/[type]/page.tsx
│   ├── fortune/page.tsx
│   ├── pay-fortune/page.tsx
│   ├── layout.tsx
│   └── page.tsx
├── components/cats-component.tsx
├── public/
├── .env.example
├── middleware.ts
├── next.config.ts
├── package.json
├── pnpm-lock.yaml
└── README.md
```

## Requirements

- Node.js 20.9 or later (Next.js 16 requirement)
- pnpm (the repository includes `pnpm-lock.yaml`)
- A Solana wallet address to receive payments
- A Coinbase Developer Platform client key for Coinbase Pay
- An OpenAI API key for the fortune-teller endpoint

## Configuration

Copy `.env.example` to `.env.local` and set the values:

| Variable | Purpose |
| --- | --- |
| `NEXT_PUBLIC_RECEIVER_ADDRESS` | Solana wallet address that receives payments. |
| `NEXT_PUBLIC_NETWORK` | `solana-devnet`, `solana-testnet`, or `solana-mainnet-beta`. |
| `NEXT_PUBLIC_FACILITATOR_URL` | Facilitator that verifies payments. `.env.example` uses `https://x402.org/facilitator`. |
| `NEXT_PUBLIC_CDP_CLIENT_KEY` | Coinbase Developer Platform client key used by the payment UI. |
| `OPENAI_API_KEY` | Server-side key used by `/api/devine`. |

```bash
cp .env.example .env.local
```

The `NEXT_PUBLIC_` prefix exposes a variable to the browser. The receiver address, network, facilitator URL, and CDP client key are therefore public configuration values. `OPENAI_API_KEY` is read only by the server route.

## Setup

```bash
pnpm install
cp .env.example .env.local
pnpm dev
```

Open `http://localhost:3000`.

## Build and Run

```bash
pnpm build
pnpm start
```

## Route Configuration

Protected routes and prices are defined in `middleware.ts`:

```typescript
const x402PaymentMiddleware = paymentMiddleware(
  address,
  {
    '/pay-fortune': {
      price: '$0.01',
      config: { description: 'AI Fortune Telling Service' },
      network,
    },
    '/content/cheap': {
      price: '$0.01',
      config: { description: 'Access to cheap content' },
      network,
    },
    '/content/expensive': {
      price: '$0.25',
      config: { description: 'Access to expensive content' },
      network,
    },
  },
  { url: facilitatorUrl },
  {
    cdpClientKey,
    appLogo: '/logos/x402-examples.png',
    appName: 'x402 Demo',
    sessionTokenEndpoint: '/api/x402/session-token',
  },
)
```

The exported `config.matcher` limits middleware execution to the same three paths.

To add a paid route:

1. Add an entry to the `paymentMiddleware` route map.
2. Add the path to `config.matcher`.
3. Create the corresponding page or route handler.

## Testing

The repository does not include automated payment tests. Verify the flow manually on devnet:

1. Start the development server with devnet configuration.
2. Request `/content/cheap` or `/content/expensive` without a session and confirm the 402 response or payment dialog.
3. Complete the payment with a funded devnet wallet.
4. Confirm the protected page loads after verification.
5. Repeat with `/pay-fortune` and confirm the success page links to `/fortune`.

Use a devnet wallet and devnet tokens. Mainnet payments are real transfers.

## Operational Notes

- **Payment boundary** - Only `/pay-fortune`, `/content/cheap`, and `/content/expensive` are matched by the middleware. `/fortune` and `/api/devine` can be requested directly without a payment session. If the OpenAI endpoint must be paid, add it to the middleware route map and matcher.
- **Session endpoint** - The middleware configuration references `/api/x402/session-token`, but this repository does not include that route. Confirm the endpoint required by the selected `x402-next` version and add it if session-token issuance is required.
- **Payment logo** - `appLogo` points to `/logos/x402-examples.png`. That file is not present under `public/`; add it or change the path.
- **Facilitator trust** - Payment verification depends on the configured facilitator. A production deployment should use a facilitator with an appropriate availability and trust model.
- **Public client key** - The CDP client key is intentionally exposed to the browser. Do not place server secrets in `NEXT_PUBLIC_` variables.

## Security Considerations

- Prices are configured in the middleware, not in page components.
- Payment verification occurs off-chain through the facilitator before the middleware grants a session.
- The receiver address and network are environment-driven; verify them before deploying to mainnet.
- The OpenAI route forwards user input directly into the prompt and has no authentication, rate limiting, or input validation. Exposing it publicly can incur API costs.
- The application does not implement user accounts, refunds, or payment-dispute handling.

## Dependencies

| Package | Purpose |
| --- | --- |
| `next` | Next.js App Router and middleware. |
| `react`, `react-dom` | UI runtime. |
| `viem` | Address and network types used by the middleware. |
| `x402-next` | X402 payment middleware. |

## References

- [X402 specification](https://github.com/coinbase/x402)
- [x402-next package](https://www.npmjs.com/package/x402-next)
- [Coinbase Developer Platform X402 documentation](https://docs.cdp.coinbase.com/x402/quickstart-for-sellers)
- [Solana documentation](https://solana.com/docs)
