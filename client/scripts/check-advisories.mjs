import { readFileSync } from "node:fs";

const exceptionsFile = new URL("../audit-exceptions.json", import.meta.url);
const failingSeverities = new Set(["high", "critical"]);
const staleExceptionsFail = process.env.GITHUB_EVENT_NAME !== "pull_request";

function readReport(path) {
  let report;
  try {
    report = JSON.parse(readFileSync(path, "utf8"));
  } catch (error) {
    throw new Error(`npm audit wrote no JSON report to ${path}: ${error.message}`);
  }
  if (report.error) {
    throw new Error(`npm audit failed: ${JSON.stringify(report.error)}`);
  }
  if (typeof report.vulnerabilities !== "object" || report.vulnerabilities === null) {
    throw new Error("npm audit reported no object vulnerabilities");
  }
  return report;
}

/** Returns each advisory once with the packages it affects, ignoring entries that are listed only through a dependency. */
function advisories(report) {
  const byId = new Map();
  for (const entry of Object.values(report.vulnerabilities)) {
    for (const advisory of entry.via.filter((via) => typeof via === "object")) {
      const id = advisory.url.replace(/^.*\//, "");
      const known = byId.get(id) ?? { id, severity: advisory.severity, title: advisory.title, packages: new Set() };
      known.packages.add(advisory.name);
      byId.set(id, known);
    }
  }
  return [...byId.values()];
}

function malformedExceptions(exceptions) {
  return exceptions
    .filter((exception) => typeof exception.reason !== "string" || !/^\d{4}-\d{2}-\d{2}$/.test(exception.expires ?? ""))
    .map((exception) => `exception for ${exception.id} needs a reason and an expiry date as YYYY-MM-DD`);
}

function problems(found, exceptions, today) {
  const excepted = new Map(exceptions.map((exception) => [exception.id, exception]));
  const unexcepted = found
    .filter((advisory) => failingSeverities.has(advisory.severity) && !excepted.has(advisory.id))
    .map((advisory) => `${advisory.id} (${advisory.severity}) in ${[...advisory.packages].join(", ")}: ${advisory.title}`);
  const expired = exceptions
    .filter((exception) => exception.expires < today)
    .map((exception) => `exception for ${exception.id} expired on ${exception.expires}`);
  return [...malformedExceptions(exceptions), ...unexcepted, ...expired];
}

function staleExceptions(found, exceptions) {
  const ids = new Set(found.map((advisory) => advisory.id));
  return exceptions
    .filter((exception) => !ids.has(exception.id))
    .map((exception) => `exception for ${exception.id} matches no advisory, remove it from audit-exceptions.json`);
}

const found = advisories(readReport(process.argv[2]));
const exceptions = JSON.parse(readFileSync(exceptionsFile, "utf8"));
const failures = problems(found, exceptions, new Date().toISOString().slice(0, 10));
const stale = staleExceptions(found, exceptions);

for (const advisory of found.filter((advisory) => !failingSeverities.has(advisory.severity))) {
  console.log(`${advisory.id} (${advisory.severity}) in ${[...advisory.packages].join(", ")}: ${advisory.title}`);
}
if (staleExceptionsFail) {
  failures.push(...stale);
} else {
  stale.forEach((message) => console.warn(`warning: ${message}`));
}
if (failures.length > 0) {
  console.error(failures.join("\n"));
  process.exit(1);
}
console.log("advisories ok");
