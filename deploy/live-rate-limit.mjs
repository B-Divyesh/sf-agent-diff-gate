#!/usr/bin/env node

import { pathToFileURL } from 'node:url';

export const RATE_LIMIT_ALLOWANCE = 40;
export const RATE_LIMIT_PROBE_SIZE = 100;

export function assertRateLimitResults(
  results,
  { allowance = RATE_LIMIT_ALLOWANCE, total = RATE_LIMIT_PROBE_SIZE } = {},
) {
  if (results.length !== total) {
    throw new Error(`Rate-limit probe returned ${results.length} responses; expected ${total}.`);
  }

  const accepted = results.filter(({ status }) => status === 200);
  const rejected = results.filter(({ status }) => status === 429);
  const unexpected = results.filter(({ status }) => status !== 200 && status !== 429);
  const missingRetryAfter = rejected.filter(({ retryAfter }) => retryAfter !== '1');

  if (accepted.length !== allowance) {
    throw new Error(
      `Rate-limit probe accepted ${accepted.length} requests; expected exactly ${allowance} from one client.`,
    );
  }
  if (rejected.length !== total - allowance) {
    throw new Error(
      `Rate-limit probe rejected ${rejected.length} requests; expected ${total - allowance}.`,
    );
  }
  if (unexpected.length) {
    throw new Error(`Rate-limit probe received unexpected statuses: ${unexpected.map(({ status }) => status).join(', ')}.`);
  }
  if (missingRetryAfter.length) {
    throw new Error(`${missingRetryAfter.length} throttled responses did not include Retry-After: 1.`);
  }

  return { accepted: accepted.length, rejected: rejected.length, retryAfter: '1' };
}

export async function probeRateLimit(baseUrl) {
  const base = new URL(baseUrl);
  const probe = `${Date.now()}-${Math.random().toString(16).slice(2)}`;

  // Start a new server-side one-second window after identity and route checks.
  await new Promise(resolve => setTimeout(resolve, 1100));
  const results = await Promise.all(
    Array.from({ length: RATE_LIMIT_PROBE_SIZE }, async (_, index) => {
      const url = new URL('/api/auth/status', base);
      url.searchParams.set('rate_probe', probe);
      url.searchParams.set('request', String(index));
      try {
        const response = await fetch(url, {
          headers: { 'cache-control': 'no-store' },
          signal: AbortSignal.timeout(15_000),
        });
        await response.arrayBuffer();
        return { status: response.status, retryAfter: response.headers.get('retry-after') };
      } catch (error) {
        return { status: 0, retryAfter: null, error: String(error) };
      }
    }),
  );
  return assertRateLimitResults(results);
}

async function main() {
  const baseUrl = process.argv[2] ?? 'https://agent-diff-gate.sociobot.in';
  const summary = await probeRateLimit(baseUrl);
  process.stdout.write(
    `Live rate limit passed: ${summary.accepted} accepted, ${summary.rejected} returned 429, and every rejection sent Retry-After: ${summary.retryAfter}.\n`,
  );
}

if (process.argv[1] && import.meta.url === pathToFileURL(process.argv[1]).href) {
  main().catch(error => {
    console.error(error.message);
    process.exitCode = 1;
  });
}
