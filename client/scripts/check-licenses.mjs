import { readFileSync } from "node:fs";

const denyFile = new URL("../../server/deny.toml", import.meta.url);
const exceptionsFile = new URL("../license-exceptions.json", import.meta.url);

function allowedLicenses() {
  const withoutComments = readFileSync(denyFile, "utf8").replace(/#.*$/gm, "");
  const list = withoutComments.match(/^allow = \[([^\]]*)\]/m);
  if (!list) {
    throw new Error(`${denyFile.pathname} has no list "allow"`);
  }
  return new Set([...list[1].matchAll(/"([^"]+)"/g)].map((match) => match[1]));
}

function tokenize(expression) {
  return expression.match(/\(|\)|[^\s()]+/g) ?? [];
}

/** Tells whether an SPDX expression of identifiers, OR, AND and parentheses is satisfied by the allowed licenses. */
function satisfies(expression, allowed) {
  const tokens = tokenize(expression);
  let position = 0;

  function fail(reason) {
    throw new Error(`${reason} in "${expression}"`);
  }

  function next() {
    return tokens[position++];
  }

  function either() {
    let result = both();
    while (tokens[position] === "OR") {
      next();
      result = both() || result;
    }
    return result;
  }

  function both() {
    let result = license();
    while (tokens[position] === "AND") {
      next();
      result = license() && result;
    }
    return result;
  }

  function license() {
    const token = next();
    if (token === "(") {
      const result = either();
      if (next() !== ")") fail("missing closing parenthesis");
      return result;
    }
    if (token === undefined || !/^[A-Za-z0-9.-]+$/.test(token)) fail(`unsupported token "${token ?? "end"}"`);
    if (token === "OR" || token === "AND") fail(`misplaced operator ${token}`);
    return allowed.has(token);
  }

  const result = either();
  if (position < tokens.length) fail(`unsupported token "${tokens[position]}"`);
  return result;
}

/** Packages of a report of `pnpm licenses list --json`, which groups the installed packages by their license. */
function entries(report) {
  return Object.entries(report).flatMap(([license, packages]) => packages.map((entry) => ({ license, name: entry.name, versions: entry.versions })));
}

/** An exception covers a package only where no production dependency needs it, so its license never reaches an artifact. */
function isExcepted(exceptions, production, { name, license }) {
  return !production.has(name) && exceptions.some((exception) => exception.name === name && exception.license === license);
}

function unusedExceptions(exceptions, packages) {
  return exceptions
    .filter((exception) => !packages.some(({ name, license }) => name === exception.name && license === exception.license))
    .map((exception) => `exception for ${exception.name} (${exception.license}) matches no package, remove it from license-exceptions.json`);
}

function problems(report, productionReport) {
  const allowed = allowedLicenses();
  const exceptions = JSON.parse(readFileSync(exceptionsFile, "utf8"));
  const production = new Set(entries(productionReport).map(({ name }) => name));
  const packages = entries(report);
  const denied = packages.flatMap((entry) => {
    const name = `${entry.name} ${entry.versions.join(", ")}`;
    if (entry.license === "Unknown") return [`${name} declares no license`];
    if (isExcepted(exceptions, production, entry)) return [];
    try {
      return satisfies(entry.license, allowed) ? [] : [`${name} has license ${entry.license}, which deny.toml does not allow`];
    } catch (error) {
      return [`${name}: ${error.message}`];
    }
  });
  return [...denied, ...unusedExceptions(exceptions, packages)];
}

const [reportFile, productionReportFile] = process.argv.slice(2);
if (!reportFile || !productionReportFile) {
  console.error("usage: check-licenses.mjs <output of pnpm licenses list --json> <the same with --prod>");
  process.exit(2);
}
const found = problems(JSON.parse(readFileSync(reportFile, "utf8")), JSON.parse(readFileSync(productionReportFile, "utf8")));
if (found.length > 0) {
  console.error(found.join("\n"));
  process.exit(1);
}
console.log("licenses ok");
