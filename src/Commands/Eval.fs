/// <summary>
/// Runs code in an emulator and displays the result.
/// </summary>
module FAQBotCC.Commands.Eval

open System
open System.Collections.Generic
open System.Net
open System.IO
open System.Net.Http
open System.Runtime.InteropServices
open System.Text.RegularExpressions
open System.Threading
open System.Threading.Tasks

open Discord
open Discord.Interactions
open Discord.Commands
open Discord.WebSocket

open Microsoft.Extensions.Logging

open FAQBotCC
open FAQBotCC.Extensions

type private CodeBlock =
  | Literal of string
  | Url of string


type Result = { message : string; attachment : Option<FileAttachment> }

module Result =
  let failure message = { message = $":bangbang: {message}"; attachment = None }


/// <summary>
/// Extracts a message from a code block and runs it.
///
/// We try to extract code in the following ways:
///  - If this message is empty, use the parent method instead.
///  - Find all code blocks or attachments in the message.
///  - If there are none, use the whole message minus the command name.
/// </summary>
type MessageEvaluator(config : Config, logger : ILogger<MessageEvaluator>, client : IDiscordClient, http : HttpClient) =

  static let codeBlock =
    Regex(
      @"```(?:lua)?\n(.*?)```|(`+)(.*?)\2",
      RegexOptions.Singleline
      ||| RegexOptions.IgnoreCase
    )

  static let dropCommand = Regex(@"^%([a-z]+)")

  let getCodeBlock result =
    task {
      match result with
      | Literal content -> return Some content
      | Url url ->
        try
          let! result = http.GetStringAsync(url)
          return Some result
        with
        | e ->
          logger.LogError(e, "Error downloading attachment {}", url)
          return None
    }

  let getCodeSnippets (message : IMessage) : IReadOnlyList<CodeBlock> =
    let codeBlocks = List<CodeBlock>()

    for attachment in message.Attachments do
      if
        attachment.ContentType <> null
        && attachment.ContentType.Contains("text/plain")
      then
        codeBlocks.Add(Url(attachment.Url))

    for captures in codeBlock.Matches(message.Content) do
      let code =
        match captures.Groups[1].Value with
        | "" -> captures.Groups[3].Value
        | x -> x
      codeBlocks.Add(Literal(code))

    if codeBlocks.Count > 0 then
      codeBlocks
    else
      let content = dropCommand.Replace(message.Content, "").Trim()
      if content.Length > 0 then [ Literal content ] else []

  member _.Screenshot(message : IMessage) =
    task {
      let mutable blocks = getCodeSnippets message
      if blocks.Count = 0 && message.Reference <> null then
        logger.LogTrace("No code blocks in original message, looking at reply.")
        match! client.GetMessageAsync message.Reference with
        | None -> ()
        | Some reply -> blocks <- getCodeSnippets reply

      if blocks.Count = 0 then
        return Result.failure "No code found in message!"
      else
        let warnings = List<string>()
        if blocks.Count > 1 then
          warnings.Add(":warning: Multiple code blocks, choosing the first.")

        match! getCodeBlock blocks[0] with
        | None -> return Result.failure "Error reading attachment."
        | Some codeBlock when codeBlock.Length > 128 * 1024 ->
          // 128K is the same length as we use on nginx.
          return Result.failure "Code block is too long to be run. Sorry!"
        | Some codeBlock ->
          logger.LogInformation($"Running %A{codeBlock}")

          try
            use cancel = new CancellationTokenSource(20_000)
            use! result = http.PostAsync(config.GetEvalServer, new StringContent(codeBlock), cancel.Token)
            match result.StatusCode with
            | HttpStatusCode.OK ->
              let hasValues, values = result.Headers.TryGetValues("X-Clean-Exit")
              if not hasValues
                 || Seq.exists (fun x -> x <> "True") values then
                warnings.Add(":warning: Computer ran for too long.")

              let! image = result.Content.ReadAsByteArrayAsync()
              return
                { message = String.Join("\n", warnings)
                  attachment = Some(new FileAttachment(new MemoryStream(image), "image.png")) }
            | code ->
              logger.LogInformation("Got HTTP Response {} instead", code)
              return Result.failure "No screenshot returned. Sorry!"
          with
          | e ->
            logger.LogError(e, "Failed to connect to {}", config.GetEvalServer)
            return Result.failure "Unknown error when running code"
    }


/// <summary>
/// The main %eval command. This runs the code and then posts a reply with the
/// screenshot and buttons to rerun and delete the code.
/// </summary>
type EvalTextCommand(evaluator : MessageEvaluator) =
  inherit ModuleBase<ICommandContext>()

  [<Command("eval")>]
  [<Alias("exec", "code")>]
  [<Summary("Runs code in this message")>]
  member this.Eval([<Remainder; Optional; DefaultParameterValue("")>] _query : string) : Task =
    task {
      let context = this.Context.DiscordContext
      match! evaluator.Screenshot this.Context.Message with
      | { message = message; attachment = None } -> return! context.Respond(message)
      | { message = message; attachment = Some attachment } ->
        let components =
          ComponentBuilder()
            .WithButton(customId = "on_rerun", label = "Rerun", style = ButtonStyle.Primary)
            .WithButton(customId = "on_delete", label = "Delete", style = ButtonStyle.Danger, emote = Emoji("🗑"))
            .Build()
        return! context.Respond(message, file = attachment, components = components)
    }

/// <summary>
/// Handlers for the rerun and delete buttons.
///
/// Both of these look up the original %eval message before running, to ensure
/// that only the "owner" can actually use these buttons.
///
/// Note that we also need to use the original message when rerunning - there's
/// no code blocks in the bot-posted message!
/// </summary>
type EvalInteractions(evaluator : MessageEvaluator) =
  inherit InteractionModuleBase<SocketInteractionContext<SocketMessageComponent>>()

  member private this.canInteract(message : IMessage) : bool =
    match this.Context.User, this.Context.Channel with
    | user, _ when message.Author = user -> true
    | IGuildUser user, IGuildChannel channel when user.GetPermissions(channel).ManageMessages -> true
    | _ -> false

  member this.GetOriginal() =
    task {
      let interaction = this.Context.Interaction in

      match! this.Context.Client.GetReferencedMessageAsync(interaction.Message) with
      | None ->
        let! () = interaction.RespondAsync("I can't remember anything about this message :/.", ephemeral = true)
        return None
      | Some message when not (this.canInteract message) ->
        let! () = interaction.RespondAsync("Only the original commenter can do this. Sorry!", ephemeral = true)
        return None
      | Some message ->
        Printf.printfn "Get original => %s" (this.Context.User.GetType().ToString())
        return Some message
    }

  [<ComponentInteraction("on_rerun")>]
  member this.Rerun() : Task =
    task {
      match! this.GetOriginal() with
      | None -> return ()
      | Some message ->
        let! () = this.Context.Interaction.DeferAsync(ephemeral = true)
        match! evaluator.Screenshot message with
        | { message = message; attachment = None } ->
          let! _ = this.Context.Interaction.FollowupAsync(text = message, ephemeral = true)
          return ()
        | { message = message; attachment = Some attachment } ->
          let! () =
            this.Context.Interaction.Message.ModifyAsync (fun props ->
              props.Content <- message
              props.Attachments <- Optional([ attachment ]))
          return ()
    }

  [<ComponentInteraction("on_delete")>]
  member this.Delete() : Task =
    task {
      match! this.GetOriginal() with
      | None -> return ()
      | Some _ ->
        let! () = this.Context.Interaction.Message.DeleteAsync()
        return! this.Context.Interaction.DeferAsync(ephemeral = true)
    }
