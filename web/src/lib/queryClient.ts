import { QueryClient } from "@tanstack/react-query";

// Live-update strategy: the backend has no push channel (no SSE/WebSocket),
// but agents tick on minute-scale cron and silently mutate tickets/comments
// out from under the UI. So we poll. 30s is a good cost/freshness balance
// for the project board and ticket views; queries that need tighter latency
// (agents deployment list, single-deployment detail) override
// `refetchInterval` locally with shorter values.
//
// `refetchIntervalInBackground: false` ensures hidden tabs stop polling.
// `refetchOnWindowFocus: true` covers the "I came back to my desk" case
// without waiting for the next tick.
export const queryClient = new QueryClient({
  defaultOptions: {
    queries: {
      staleTime: 30_000,
      gcTime: 5 * 60_000,
      retry: (failureCount, error) => {
        // Don't retry 4xx; let auth handler kick in for 401.
        const anyErr = error as { status?: number };
        if (anyErr?.status && anyErr.status >= 400 && anyErr.status < 500) return false;
        return failureCount < 2;
      },
      refetchOnWindowFocus: true,
      refetchOnReconnect: true,
      refetchInterval: 30_000,
      refetchIntervalInBackground: false,
    },
    mutations: {
      retry: false,
    },
  },
});
