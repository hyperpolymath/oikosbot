// ReScript bindings for Deno runtime APIs

module Env = {
  @val @scope(("Deno", "env")) external get: string => option<string> = "get"
  @val @scope(("Deno", "env")) external set: (string, string) => unit = "set"
}

module Http = {
  type request
  type response

  type connInfo = {remoteAddr: {hostname: string, port: int}}

  type handler = (request, connInfo) => promise<response>

  type serveOptions = {
    port?: int,
    hostname?: string,
    onListen?: {hostname: string, port: int} => unit,
  }

  @send external url: request => string = "url"
  @send external method_: request => string = "method"
  @send external headers: request => Js.Dict.t<string> = "headers"
  @send external json: request => promise<Js.Json.t> = "json"
  @send external text: request => promise<string> = "text"

  @new @scope("globalThis")
  external makeResponse: (string, {..}) => response = "Response"

  @new @scope("globalThis")
  external makeJsonResponse: (string, {..}) => response = "Response"

  @val @scope("Deno")
  external serve: (serveOptions, handler) => unit = "serve"
}

module Crypto = {
  type subtleCrypto

  @val @scope(("globalThis", "crypto"))
  external subtle: subtleCrypto = "subtle"

  @send
  external importKey: (
    subtleCrypto,
    string,
    Js.TypedArray2.Uint8Array.t,
    {..},
    bool,
    array<string>,
  ) => promise<'key> = "importKey"

  @send
  external sign: (subtleCrypto, string, 'key, Js.TypedArray2.Uint8Array.t) => promise<'signature> =
    "sign"

  @send
  external verify: (
    subtleCrypto,
    string,
    'key,
    Js.TypedArray2.Uint8Array.t,
    Js.TypedArray2.Uint8Array.t,
  ) => promise<bool> = "verify"
}

module TextEncoder = {
  type t

  @new @scope("globalThis") external make: unit => t = "TextEncoder"
  @send external encode: (t, string) => Js.TypedArray2.Uint8Array.t = "encode"
}

module TextDecoder = {
  type t

  @new @scope("globalThis") external make: unit => t = "TextDecoder"
  @send external decode: (t, Js.TypedArray2.Uint8Array.t) => string = "decode"
}
