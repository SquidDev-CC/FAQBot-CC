module FAQBotCC.Discord

open System
open System.Diagnostics
open System.Threading.Tasks

open Discord
open Discord.Commands
open Discord.Interactions
open Discord.WebSocket

open Microsoft.Extensions.DependencyInjection
open Microsoft.Extensions.Logging

open FAQBotCC.Extensions


let private commandTime =
  Telemetry.metricsSource.CreateHistogram<int64>(
    name = "faqcc.command_time",
    description = "Time taken to execute a command"
  )


/// <summary>Start a new trace/activity and add some information from Discord.</summary>
let private startCommand (name : string) (user : IUser) (guild : IGuild) =
  let activity = Telemetry.activitySource.StartActivity(name, ActivityKind.Server)
  activity.SetTag' "discord.user_name" user.Username
  activity.SetTag' "discord.user_tag" user.Mention
  if guild <> null then activity.SetTag' "discord.guild" guild.Name
  activity


let private log (logger : ILogger<_>) (msg : LogMessage) =
  let level =
    match msg.Severity with
    | LogSeverity.Critical -> LogLevel.Critical
    | LogSeverity.Error -> LogLevel.Error
    | LogSeverity.Warning -> LogLevel.Warning
    | LogSeverity.Info -> LogLevel.Information
    | LogSeverity.Verbose -> LogLevel.Debug
    | LogSeverity.Debug -> LogLevel.Trace
    | _ -> LogLevel.Information

  use _ = logger.BeginScope(msg.Source)
  logger.Log(level, msg.Exception, "{}", msg.Message)
  Task.CompletedTask


let makeClient (services : IServiceProvider) =
  let logger = services.GetRequiredService<ILogger<DiscordSocketClient>>()

  let socketConfig = new DiscordSocketConfig()
  socketConfig.LogLevel <- LogSeverity.Debug
  socketConfig.MessageCacheSize <- 250
  socketConfig.GatewayIntents <-
    GatewayIntents.Guilds
    ||| GatewayIntents.GuildMessages
    ||| GatewayIntents.DirectMessages
  let client = new DiscordSocketClient(socketConfig)
  client.add_Log (log logger)
  client


/// <summary>
/// Discord.Net by default runs tasks from the main Gateway event loop, rather than
/// asynchronously. While there /is/ an async way to run them, it doesn't
/// play well with the our tracing/metrics code, so we roll our own.
/// </summary>
let private runAsync (logger : ILogger) (fn : 'a -> Task) input =
  let task = fn input
  let _background =
    task.ContinueWith(
      (fun output ->
        if output.IsFaulted then
          let e = output.Exception
          let e = if e.InnerExceptions.Count = 0 then e.InnerExceptions[0] else e
          logger.LogError(e, "Error running command or interaction")
        else if output.IsCanceled then
          logger.LogWarning("Error running async task")
        else
          logger.LogTrace("Background task complete")),
      TaskScheduler.Default
    )
  Task.CompletedTask


let private subscribeAsync subscribe logger fn = subscribe (runAsync logger fn)


let makeCommands (services : IServiceProvider) =
  let client = services.GetRequiredService<DiscordSocketClient>()
  let logger = services.GetRequiredService<ILogger<CommandService>>()

  let commandConfig = new CommandServiceConfig()
  commandConfig.LogLevel <- LogSeverity.Debug
  commandConfig.CaseSensitiveCommands <- false

  let commands = new CommandService(commandConfig)
  commands.add_Log (log logger)

  subscribeAsync client.add_MessageReceived logger (fun msg ->
    let mutable argPos = 0

    match msg with
    | :? SocketUserMessage as msg when
      not msg.Author.IsBot
      && (msg.HasCharPrefix('%', &argPos)
          || msg.HasMentionPrefix(client.CurrentUser, &argPos))
      ->
      task {
        let context = SocketCommandContext(client, msg)

        use activity = startCommand "message" msg.Author context.Guild
        use _ = commandTime.Timed(tag "command" "unknown", tag "mode" "message")

        logger.LogInformation("Running {} (started by {})", msg.Content, msg.Author.Username)
        let! result = commands.ExecuteAsync(context, argPos, services)
        activity.SetOk(result.IsSuccess)
        return result
      }
    | _ -> Task.CompletedTask)

  commands.add_CommandExecuted (fun command context result ->
    if not command.IsSpecified || result.IsSuccess then
      Task.CompletedTask
    else
      let context : IDiscordContext = context.DiscordContext

      match ValueOption.ofNullable result.Error with
      | ValueSome CommandError.BadArgCount -> context.Respond($"Command expected more arguments.")
      | _ -> context.Respond($"Error handling this command ({result})"))

  commands


let makeInteractions (services : IServiceProvider) =
  let client = services.GetRequiredService<DiscordSocketClient>()
  let config = services.GetRequiredService<Config>()
  let logger = services.GetRequiredService<ILogger<InteractionService>>()

  let interactionConfig = new InteractionServiceConfig()
  interactionConfig.LogLevel <- LogSeverity.Debug
  let interactions = new InteractionService(client, interactionConfig)
  interactions.add_Log (log logger)

  client.add_Ready (fun () ->
    match config.GuildId with
    | None -> interactions.RegisterCommandsGloballyAsync()
    | Some guild -> interactions.RegisterCommandsToGuildAsync(guild))

  subscribeAsync client.add_InteractionCreated logger (fun msg ->
    task {
      // We want to pass the most accurate context we can do the command, so we do this terribly ugly dispatch.
      // It would be possible to subscribe to all the events separately, but this ends up being a little easier.
      let kind, name, (context : IInteractionContext) =
        match msg with
        | :? SocketSlashCommand as cmd -> "slash", cmd.CommandName, SocketInteractionContext<_>(client, cmd)
        | :? SocketUserCommand as cmd -> "user_command", cmd.CommandName, SocketInteractionContext<_>(client, cmd)
        | :? SocketMessageCommand as cmd -> "message_command", cmd.CommandName, SocketInteractionContext<_>(client, cmd)
        | :? SocketMessageComponent as cmd ->
          "message_component", cmd.Data.CustomId, SocketInteractionContext<_>(client, cmd)
        | :? SocketAutocompleteInteraction as interaction ->
          "autocomplete", interaction.Data.CommandName, SocketInteractionContext<_>(client, interaction)
        | :? SocketModal as modal -> "modal", modal.Data.CustomId, SocketInteractionContext<_>(client, modal)
        | _ -> "interaction", "unknown", SocketInteractionContext<_>(client, msg)
      use activity = startCommand $"{name}.{kind}" msg.User context.Guild
      use _ = commandTime.Timed(tag "command" name, tag "mode" kind)

      logger.LogInformation("Running {} {} (started by {})", name, kind, msg.User.Username)
      let! result = interactions.ExecuteCommandAsync(context, services)
      activity.SetOk(result.IsSuccess)
      return ()
    })

  interactions
