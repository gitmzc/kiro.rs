# Frontend Implementation Plan for kiro-rs

## Technology Stack
- **Frontend Framework**: React 19 + TypeScript + Vite (Modern, rich ecosystem, optimized build).
- **UI Library**: shadcn/ui + Tailwind CSS (Professional look, extremely lightweight CSS output).
- **State Management**: TanStack Query (React Query) (Best for server-state synchronization).
- **Embedding**: `rust-embed` (Embeds the `dist/` folder into the Rust binary at compile time).
- **Charts**: Recharts (Lightweight SVG charts).

## Pages
1.  **Dashboard**:
    - Status Cards (Uptime, Version, Active Credential).
    - Charts (Requests/Hour, Token Usage).
2.  **Credentials**:
    - List view with priority sorting.
    - Status badges (Active, Rate Limited, Expired).
    - Add/Edit/Delete/Refresh actions.
3.  **Logs**:
    - Real-time SSE stream viewer.
    - Log level filtering.
4.  **Settings**:
    - System config (Port, Host, Mock parameters).

## API Structure (`/admin/api/v1`)
- `GET /stats`: System overview.
- `GET/POST/DELETE /credentials`: Management.
- `GET /logs/stream`: SSE Log stream.
- `GET/PUT /config`: Configuration management.
