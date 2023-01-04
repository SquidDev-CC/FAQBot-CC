namespace FAQBotCC

open System.IO
open System.Text.Json.Serialization
open System.Text.Json


type Config =
  { /// <summary>Token to connect to Discord with.</summary>
    [<JsonPropertyName("token")>]
    Token : string

    /// <summary>Restricted guild id this bot registers commands under.</summary>
    [<JsonPropertyName("guild_id")>]
    GuildId : Option<uint64>

    /// <summary>Export metrics and traces via OTLP. </summary>
    [<JsonPropertyName("metrics")>]
    Metrics : Option<bool>

    /// <summary>
    /// The server to use for evaling code. By default eval.tweaked.cc, but may be a custom server.
    /// </summary>
    [<JsonPropertyName("eval_server")>]
    EvalServer : Option<string> }


type Config with
  member this.GetEvalServer =
    Option.defaultValue "https://eval.tweaked.cc" this.EvalServer

  member this.HasMetrics = Option.defaultValue false this.Metrics

  static member FromFile() =
    if File.Exists "config.json" then
      let contents = File.ReadAllText("config.json")
      JsonSerializer.Deserialize<Config> contents
    else
      let token = File.ReadAllText("token").Trim()
      { Token = token; GuildId = None; Metrics = None; EvalServer = None }
