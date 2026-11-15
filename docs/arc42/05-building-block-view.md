# Building block view

## Whitebox overall system

Adunata is one binary. The server modules answer API requests and serve the browser client, which the binary embeds. The client runs in the browser and talks to the server only through the API.

```mermaid
flowchart LR
  subgraph browser[Browser]
    client[Client]
  end
  subgraph binary[Adunata binary]
    api[api]
    clientModule[client]
    configuration[configuration]
    database[database]
    health[health]
    legal[legal]
    localization[localization]
    mail[mail]
    members[members]
    spike[spike]
    serving[serving]
    validation[validation]
  end
  client -- JSON over HTTP --> api
  client -- loads files --> clientModule
  api --> spike
  api --> localization
  api --> health
  api --> legal
  api --> members
  health --> database
  members --> database
  members --> localization
  members --> validation
  spike --> validation
  mail --> members
  spike --> database
  spike --> mail
  spike --> localization
  mail --> database
  database --> postgres[(PostgreSQL)]
  mail --> smtp[Mail server]
```

|Building block|Responsibility|Interface|
|-|-|-|
|Client|The single-page application in the browser. It renders pages, validates forms and shows text in the reader's language.|Calls the API. Loads its catalog from the server.|
|api|Collects the routes of all modules and describes them as one OpenAPI document.|The HTTP router and the OpenAPI document.|
|client|Serves the files of the built client from the binary. Paths without a file extension get the client's entry page.|The HTTP router for all paths outside the API.|
|configuration|Reads the settings of the instance from the environment.|The settings as one value.|
|database|Opens the connection pool and applies pending migrations. It refuses a schema that is newer than the binary.|A connected pool.|
|health|Reports whether the instance can serve requests and which dependency is unavailable.|The route `/api/health`.|
|legal|Exposes the addresses of the privacy notice and the imprint that every page links.|The route `/api/legal-links`.|
|localization|Holds one Fluent catalog per language, formats messages and serves catalogs to the client.|Formatting of messages and date ranges. The catalog route.|
|mail|Stores mail in the outbox and sends it in the background with retries.|Adding a mail within a transaction and running the delivery.|
|members|Keeps the member list with the organizer role. It refuses every change that would leave the instance without an active organizer, and deleting a member deletes everything that references the member.|The member routes and functions that other modules use to read members.|
|serving|Accepts connections and lets requests in progress finish when the service manager stops the process.|Serving the router until a termination signal arrives.|
|spike|An example form that exercises the stack from client to mail. Feature modules replace it.|The route for storing a response.|
|validation|Describes rejected fields the same way for every module.|The types of a field error and the response with status 422.|

The rules for module boundaries are in [ADR 0007](../decisions/0007-modular-monolith.md).
