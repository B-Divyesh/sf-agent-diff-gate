import { writeFile } from 'node:fs/promises';

const url = 'https://agent-diff-gate.sociobot.in/api/auth/status';
const requests = 240;
const forwardedIp = process.env.QA_FORWARDED_IP ?? '198.51.100.214';
const responses = await Promise.all(Array.from({ length: requests }, async (_, index) => {
  const response = await fetch(url, { headers: { 'X-Forwarded-For': forwardedIp } });
  await response.arrayBuffer();
  return { index, status: response.status, retry_after: response.headers.get('retry-after') };
}));
const counts = Object.fromEntries([...new Set(responses.map(({ status }) => status))].map(status => [status, responses.filter(response => response.status === status).length]));
const limited = responses.filter(({ status }) => status === 429);
const result = {
  url,
  forwarded_ip: forwardedIp,
  requests,
  status_counts: counts,
  all_429_have_retry_after: limited.length > 0 && limited.every(({ retry_after }) => retry_after),
  retry_after_values: [...new Set(limited.map(({ retry_after }) => retry_after))],
  note: 'One parallel burst carrying one explicit first-hop X-Forwarded-For value.',
};
await writeFile('.factory/evidence/verification-13/live-rate-limit-240.json', `${JSON.stringify({ ...result, responses }, null, 2)}\n`);
console.log(JSON.stringify(result, null, 2));
