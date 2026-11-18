import { type ChildProcess, execFileSync, spawn } from 'node:child_process'
import { createHash } from 'node:crypto'
import { once } from 'node:events'
import { mkdirSync, openSync, writeFileSync } from 'node:fs'
import { createServer } from 'node:net'
import path from 'node:path'

const client = path.resolve(import.meta.dirname, '..')
const root = git('rev-parse', '--show-toplevel')
const project = `adunata-e2e-${createHash('sha256').update(root).digest('hex').slice(0, 12)}`
const results = path.join(client, 'test-results')

function git(...args: string[]): string {
  return execFileSync('git', args, { cwd: client, encoding: 'utf8' }).trim()
}

function compose(...args: string[]): string {
  return execFileSync('docker', ['compose', '-p', project, ...args], {
    cwd: root,
    encoding: 'utf8',
    env: { ...process.env, POSTGRES_PORT: '0', MAILPIT_SMTP_PORT: '0', MAILPIT_UI_PORT: '0' },
    stdio: ['ignore', 'pipe', 'inherit'],
    maxBuffer: 64 * 1024 * 1024,
  }).trim()
}

/** The host port that Compose bound to a port of a service, such as 55001 for 127.0.0.1:55001. */
function hostPort(service: string, port: number): number {
  return Number(compose('port', service, String(port)).split(':').pop())
}

function freePort(): Promise<number> {
  return new Promise((resolve, reject) => {
    const server = createServer()
    server.once('error', reject)
    server.listen(0, '127.0.0.1', () => {
      const address = server.address()
      server.close(() =>
        typeof address === 'object' && address ? resolve(address.port) : reject(new Error('no free port')),
      )
    })
  })
}

/** The binary from ADUNATA_E2E_SERVER, otherwise a release build of the client and the server in this checkout. */
function serverBinary(): string {
  const given = process.env.ADUNATA_E2E_SERVER
  if (given) return path.resolve(given)
  execFileSync('pnpm', ['run', 'build'], { cwd: client, stdio: 'inherit' })
  execFileSync('cargo', ['build', '--release', '--locked'], { cwd: path.join(root, 'server'), stdio: 'inherit' })
  return path.join(root, 'server', 'target', 'release', 'adunata')
}

/** Settles with an error once the server fails to start or exits. */
function death(server: ChildProcess): Promise<never> {
  const died = new Promise<never>((_, reject) => {
    server.once('error', reject)
    server.once('exit', (code, signal) =>
      reject(new Error(`the server exited with ${code ?? signal}, see ${results}/server.log`)),
    )
  })
  died.catch(() => undefined)
  return died
}

async function untilAnswers(url: string, died: Promise<never>): Promise<void> {
  for (let attempt = 0; attempt < 300; attempt++) {
    const answer = await Promise.race([fetch(url).catch(() => undefined), died])
    if (answer?.ok) return
    await Promise.race([new Promise((resolve) => setTimeout(resolve, 100)), died])
  }
  throw new Error(`the server did not answer at ${url}, see ${results}/server.log`)
}

async function stop(server: ChildProcess | undefined): Promise<void> {
  if (!server?.pid || server.exitCode !== null || server.signalCode !== null) return
  const exited = once(server, 'exit')
  server.kill()
  await exited
}

/** Keeps the output of the services in test-results and removes the project, also when the output cannot be kept. */
function removeServices(): void {
  try {
    writeFileSync(path.join(results, 'services.log'), compose('logs', '--no-color'))
  } finally {
    compose('down', '--volumes')
  }
}

function startServer(binary: string, postgres: number, smtp: number, http: number): ChildProcess {
  mkdirSync(results, { recursive: true })
  const log = openSync(path.join(results, 'server.log'), 'w')
  return spawn(binary, ['serve'], {
    cwd: results,
    stdio: ['ignore', log, log],
    env: {
      ADNT_DATABASE_URL: `postgres://adunata:adunata@127.0.0.1:${postgres}/adunata`,
      ADNT_HTTP_LISTEN_ADDRESS: `127.0.0.1:${http}`,
      ADNT_HTTP_PUBLIC_URL: `http://localhost:${http}`,
      ADNT_SMTP_HOST: '127.0.0.1',
      ADNT_SMTP_PORT: String(smtp),
      ADNT_SMTP_ENCRYPTION: 'none',
      ADNT_MAIL_SENDER: 'Adunata <adunata@example.org>',
      ADNT_LOCALIZATION_TIME_ZONE: 'Europe/Berlin',
      ADNT_BOOTSTRAP_ORGANIZER_MAIL: 'organizer@example.org',
      ADNT_BOOTSTRAP_ORGANIZER_FIRST_NAME: 'Ada',
      ADNT_BOOTSTRAP_ORGANIZER_LAST_NAME: 'Lovelace',
    },
  })
}

/** Starts fresh services from compose.yaml as a Compose project of this checkout and a server with a cleared environment, and removes both afterwards. */
export default async function globalSetup(): Promise<() => Promise<void>> {
  const binary = serverBinary()
  compose('down', '--volumes')
  let server: ChildProcess | undefined
  const teardown = async () => {
    try {
      await stop(server)
    } finally {
      removeServices()
    }
  }
  try {
    compose('up', '--detach', '--wait')
    const http = await freePort()
    server = startServer(binary, hostPort('postgres', 5432), hostPort('mailpit', 1025), http)
    await untilAnswers(`http://127.0.0.1:${http}/api/health`, death(server))
    process.env.ADUNATA_E2E_URL = `http://localhost:${http}`
    process.env.ADUNATA_E2E_MAILPIT_URL = `http://127.0.0.1:${hostPort('mailpit', 8025)}`
    return teardown
  } catch (error) {
    await teardown().catch((cleanup: unknown) => console.error('could not remove the services:', cleanup))
    throw error
  }
}
