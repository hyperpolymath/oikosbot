// Eco-Bot Main Entry Point
// ReScript + Deno implementation

open Deno
open Types

// Logger helper
let log = (level: string, msg: string, data: option<Js.Json.t>) => {
  let timestamp = Js.Date.toISOString(Js.Date.make())
  let logObj = switch data {
  | Some(d) =>
    Js.Dict.fromArray([
      ("timestamp", Js.Json.string(timestamp)),
      ("level", Js.Json.string(level)),
      ("message", Js.Json.string(msg)),
      ("data", d),
    ])
  | None =>
    Js.Dict.fromArray([
      ("timestamp", Js.Json.string(timestamp)),
      ("level", Js.Json.string(level)),
      ("message", Js.Json.string(msg)),
    ])
  }
  Js.Console.log(Js.Json.stringify(Js.Json.object_(logObj)))
}

let info = (msg, ~data=?) => log("info", msg, data)
let error = (msg, ~data=?) => log("error", msg, data)

// JSON response helper
let jsonResponse = (data: Js.Json.t, ~status=200): Http.response => {
  Http.makeJsonResponse(
    Js.Json.stringify(data),
    {
      "status": status,
      "headers": {"Content-Type": "application/json"},
    },
  )
}

// Handle GitHub webhook
let handleGitHubWebhook = async (
  config: config,
  headers: Js.Dict.t<string>,
  body: string,
): Http.response => {
  // Verify signature if secret is configured
  switch config.githubWebhookSecret {
  | Some(secret) =>
    let signature = Js.Dict.get(headers, "x-hub-signature-256")->Belt.Option.getWithDefault("")
    let valid = await Webhook.verifyGitHubSignature(body, signature, secret)
    if !valid {
      error("Invalid GitHub webhook signature")
      return Http.makeResponse(`{"error": "Invalid signature"}`, {"status": 401})
    }
  | None => ()
  }

  // Parse payload
  let payload = try {
    Js.Json.parseExn(body)
  } catch {
  | _ => return Http.makeResponse(`{"error": "Invalid JSON"}`, {"status": 400})
  }

  let event = Webhook.parseGitHubEvent(headers, payload)

  switch event {
  | Some(e) =>
    info(
      `GitHub event: ${e.eventType}`,
      ~data=Js.Json.object_(
        Js.Dict.fromArray([
          ("repo", Js.Json.string(`${e.repository.owner}/${e.repository.name}`)),
          ("action", Js.Json.string(e.action->Belt.Option.getWithDefault(""))),
        ]),
      ),
    )

    // Handle pull request events
    if e.eventType == "pull_request" {
      let action = e.action->Belt.Option.getWithDefault("")
      if action == "opened" || action == "synchronize" {
        // In production, would trigger analysis and post comment
        let analysis = Analysis.mockAnalysis()
        let comment = Report.generatePRComment(analysis, config.mode)
        info(`Generated PR comment`, ~data=Js.Json.string(comment))
      }
    }

    jsonResponse(Js.Json.object_(Js.Dict.fromArray([("status", Js.Json.string("processed"))])))
  | None =>
    error("Failed to parse GitHub event")
    Http.makeResponse(`{"error": "Invalid event"}`, {"status": 400})
  }
}

// Handle GitLab webhook
let handleGitLabWebhook = async (
  config: config,
  headers: Js.Dict.t<string>,
  body: string,
): Http.response => {
  // Verify token if secret is configured
  switch config.gitlabWebhookSecret {
  | Some(secret) =>
    let token = Js.Dict.get(headers, "x-gitlab-token")->Belt.Option.getWithDefault("")
    if !Webhook.verifyGitLabToken(token, secret) {
      error("Invalid GitLab webhook token")
      return Http.makeResponse(`{"error": "Invalid token"}`, {"status": 401})
    }
  | None => ()
  }

  // Parse payload
  let payload = try {
    Js.Json.parseExn(body)
  } catch {
  | _ => return Http.makeResponse(`{"error": "Invalid JSON"}`, {"status": 400})
  }

  let event = Webhook.parseGitLabEvent(headers, payload)

  switch event {
  | Some(e) =>
    info(
      `GitLab event: ${e.eventType}`,
      ~data=Js.Json.object_(
        Js.Dict.fromArray([("repo", Js.Json.string(`${e.repository.owner}/${e.repository.name}`))]),
      ),
    )

    // Handle merge request events
    if e.eventType == "Merge Request Hook" {
      let analysis = Analysis.mockAnalysis()
      let comment = Report.generatePRComment(analysis, config.mode)
      info(`Generated MR comment`, ~data=Js.Json.string(comment))
    }

    jsonResponse(Js.Json.object_(Js.Dict.fromArray([("status", Js.Json.string("processed"))])))
  | None =>
    error("Failed to parse GitLab event")
    Http.makeResponse(`{"error": "Invalid event"}`, {"status": 400})
  }
}

// Main request handler
let handler = (config: config): Http.handler => {
  async (req, _connInfo) => {
    let url = Http.url(req)
    let method = Http.method_(req)
    let path = Js.String.replace(url, ~search=Js.Re.fromString("^https?://[^/]+"), ~replacement="")

    // Route requests
    switch (method, path) {
    | ("GET", "/health") =>
      jsonResponse(
        Js.Json.object_(
          Js.Dict.fromArray([
            ("status", Js.Json.string("healthy")),
            ("mode", Js.Json.string(Config.modeToString(config.mode))),
          ]),
        ),
      )

    | ("POST", "/webhooks/github") =>
      let body = await Http.text(req)
      let headers = Http.headers(req)
      await handleGitHubWebhook(config, headers, body)

    | ("POST", "/webhooks/gitlab") =>
      let body = await Http.text(req)
      let headers = Http.headers(req)
      await handleGitLabWebhook(config, headers, body)

    | ("GET", "/metrics") =>
      // OpenTelemetry metrics endpoint
      jsonResponse(
        Js.Json.object_(
          Js.Dict.fromArray([
            ("eco_bot_requests_total", Js.Json.number(0.0)),
            ("eco_bot_analyses_total", Js.Json.number(0.0)),
          ]),
        ),
      )

    | _ => Http.makeResponse("Not Found", {"status": 404})
    }
  }
}

// Main entry point
let main = () => {
  switch Config.load() {
  | Ok(config) =>
    info(
      `Starting Eco-Bot`,
      ~data=Js.Json.object_(
        Js.Dict.fromArray([
          ("port", Js.Json.number(Belt.Int.toFloat(config.port))),
          ("mode", Js.Json.string(Config.modeToString(config.mode))),
        ]),
      ),
    )

    Http.serve(
      {
        port: config.port,
        onListen: ({hostname, port}) => {
          info(`Server listening on ${hostname}:${Belt.Int.toString(port)}`)
        },
      },
      handler(config),
    )

  | Error(e) =>
    error(`Failed to load config: ${e}`)
  }
}

// Run
main()
