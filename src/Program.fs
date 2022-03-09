open System.Reflection
open System.Threading
open System.Threading.Tasks
open System.Net.Http

open Discord
open Discord.Commands
open Discord.Interactions
open Discord.WebSocket

open Microsoft.Extensions.DependencyInjection
open Microsoft.Extensions.Logging

open FAQBotCC

(task {
  let config = Config.FromFile()

  use services =
    ServiceCollection()
      .AddSingleton(config)
      .AddLogging(fun builder ->
        builder
          .Configure(fun x ->
            x.ActivityTrackingOptions <-
              ActivityTrackingOptions.SpanId
              ||| ActivityTrackingOptions.TraceId)
          .AddFilter("", LogLevel.Trace)
          .AddSimpleConsole(fun x ->
            x.IncludeScopes <- true
            x.SingleLine <- true)
        |> ignore)
      .AddHttpClient()
      .AddSingleton<DiscordSocketClient>(Discord.makeClient)
      .AddSingleton<IDiscordClient>(fun x -> x.GetRequiredService<DiscordSocketClient>() :> IDiscordClient)
      .AddSingleton<CommandService>(Discord.makeCommands)
      .AddSingleton<InteractionService>(Discord.makeInteractions)
      // Our own services
      .AddSingleton<Commands.Docs.MethodStore>()
      .AddSingleton<Commands.Eval.MessageEvaluator>()
      .BuildServiceProvider()

  use _ = Telemetry.makeTracerProvider config
  use _ = Telemetry.makeMetricsProvider config

  let client = services.GetRequiredService<DiscordSocketClient>()
  let commands = services.GetRequiredService<CommandService>()
  let interactions = services.GetRequiredService<InteractionService>()

  let! _ = commands.AddModulesAsync(Assembly.GetEntryAssembly(), services)
  let! _ = interactions.AddModulesAsync(Assembly.GetEntryAssembly(), services)
  let! _ = interactions.CreateModuleAsync("FaqInteractionCommand", services, Commands.Faq.getSlashCommands)

  let! () = client.LoginAsync(TokenType.Bot, config.Token)
  let! () = client.StartAsync()
  let! () = Task.Delay(Timeout.Infinite)
  ()
})
  .GetAwaiter()
  .GetResult()
