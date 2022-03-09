/// <summary>
/// Provides /docs and /source commands to search the CC:T and Lua docs for a
/// particular method.
///
/// We also provide auto-completion for these methods sources, as well as legacy
/// %docs and %source commands.
/// <summary>
module FAQBotCC.Commands.Docs

open System.Collections.Generic
open System.Globalization
open System.Net.Http
open System.Text.Json
open System.Text.Json.Serialization
open System.Threading.Tasks

open Discord
open Discord.Interactions
open Discord.Commands

open Microsoft.Extensions.Logging

open FAQBotCC
open FAQBotCC.Extensions


type private ArgSummaryAttribute = Interactions.SummaryAttribute


/// <summary>A documented item inside the illuaminate doc export</summary>
type DocumentedItem =
  { [<JsonPropertyName("name")>]
    Name : string
    [<JsonPropertyName("source")>]
    Source : string
    [<JsonPropertyName("summary")>]
    Summary : Option<string>
    [<JsonPropertyName("module")>]
    Module : string
    [<JsonPropertyName("module-kind")>]
    ModuleKind : string
    [<JsonPropertyName("url")>]
    Url : string }


module DocumentedItem =
  let makeEmbed (item : DocumentedItem) link =
    let embed = EmbedBuilder()
    embed.Title <- item.Name
    embed.Url <- link item
    match item.Summary with
    | None -> ()
    | Some summary -> embed.Description <- summary
    embed.Build()

  let linkDocs (item : DocumentedItem) = $"https://tweaked.cc/{item.Url}"
  let linkSource (item : DocumentedItem) = item.Source


/// <summary>Wrapper around a CachedRequest which is used with DI.</summary>
type MethodStore(logger : ILogger<MethodStore>, client : HttpClient) =
  let cache : CachedRequest.t<IReadOnlyDictionary<_, _>> =
    CachedRequest.make client logger "https://tweaked.cc/index.json" (int64 (60 * 1000)) (fun x ->
      JsonSerializer.Deserialize<Dictionary<string, DocumentedItem>>(x))

  member _.Get = CachedRequest.get cache


/// <summary>
/// Provides autocompletion for a method name.
///
/// For now, this just works off a .StartsWith check, and scores based on length.
/// </summary>
type MethodNameAutocompleteHandler(methods : MethodStore) =
  inherit AutocompleteHandler()

  override _.GenerateSuggestionsAsync
    (
      _context : IInteractionContext,
      autocompleteInteraction : IAutocompleteInteraction,
      _parameter : IParameterInfo,
      _services : System.IServiceProvider
    ) : Task<AutocompletionResult> =
    task {
      let value = autocompleteInteraction.Data.Current.Value.ToString().ToLowerInvariant()
      let! methods = methods.Get
      return
        Seq.append methods.Keys LuaNames.vars.Keys
        |> Seq.filter (fun name -> name.StartsWith(value, ignoreCase = true, culture = CultureInfo.InvariantCulture))
        |> Seq.sortBy (fun x -> x.Length)
        |> Seq.truncate 25
        |> Seq.map (fun name -> AutocompleteResult(name, name))
        |> AutocompletionResult.FromSuccess
    }


let private run (context : IDiscordContext) (methods : MethodStore) (query : string) link : Task =
  task {
    let! methods = methods.Get
    let query = query.TrimEnd('(', ')', ' ')
    // First attempt to find a CC:T method.
    match Lookup.find methods query with
    | Lookup.Exact value -> return! context.Respond(embed = DocumentedItem.makeEmbed value link)
    | Lookup.Fuzzy (name, item) ->
      return!
        context.Respond(
          text = $"Cannot find '{query}', using '{name}' instead.",
          embed = DocumentedItem.makeEmbed item link
        )
    | Lookup.Missing ->
      // Failing that, find a built-in Lua one.
      let luaQuery = query.ToLowerInvariant()
      match Lookup.find LuaNames.vars luaQuery with
      | Lookup.Exact link -> return! context.Respond(embed = EmbedBuilder().WithTitle(luaQuery).WithUrl(link).Build())
      | Lookup.Fuzzy (name, link) ->
        return!
          context.Respond(
            text = $"Cannot find '{query}', using '{name}' instead.",
            embed = EmbedBuilder().WithTitle(name).WithUrl(link).Build()
          )
      | Lookup.Missing ->
        return!
          context.Respond(
            text =
              $"Cannot find '{query}'. Please check your spelling, or contribute to the documentation at https://github.com/cc-tweaked/CC-Tweaked."
          )
  }


module private Descriptions =
  [<Literal>]
  let arg = "The function's name"

  [<Literal>]
  let docs =
    "Searches for a function with the current name and returns its documentation."

  [<Literal>]
  let source =
    "Searches for a function with the current name, and returns a link to its source code."


type DocsTextCommand(methods : MethodStore) =
  inherit ModuleBase<ICommandContext>()

  [<Command("docs")>]
  [<Alias("d")>]
  [<Summary(Descriptions.docs)>]
  member this.Docs([<Summary(Descriptions.arg)>] name : string) =
    run this.Context.DiscordContext methods name DocumentedItem.linkDocs

  [<Command("source")>]
  [<Alias("s")>]
  [<Summary(Descriptions.source)>]
  member this.Source([<Summary(Descriptions.arg)>] name : string) =
    run this.Context.DiscordContext methods name DocumentedItem.linkSource


type DocsInteractionCommand(methods : MethodStore) =
  inherit InteractionModuleBase<IInteractionContext>()

  [<SlashCommand("docs", Descriptions.docs)>]
  member this.Docs
    ([<ArgSummary(description = Descriptions.arg); Autocomplete(typedefof<MethodNameAutocompleteHandler>)>] name : string)
    =
    run this.Context.DiscordContext methods name DocumentedItem.linkDocs

  [<SlashCommand("source", Descriptions.source)>]
  member this.Source
    ([<ArgSummary(description = Descriptions.arg); Autocomplete(typedefof<MethodNameAutocompleteHandler>)>] name : string)
    =
    run this.Context.DiscordContext methods name DocumentedItem.linkSource
