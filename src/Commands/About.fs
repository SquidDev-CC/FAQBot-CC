/// <summary>
/// Provides a brief bit of information about the bot and its current state.
/// </summary>

module FAQBotCC.Commands.About

open System
open System.Threading.Tasks

open Discord
open Discord.Interactions
open Discord.Commands

open FAQBotCC
open FAQBotCC.Faqs
open FAQBotCC.Extensions


[<Literal>]
let private description =
  "Shows information about the bot as well as the relevant version numbers, uptime and useful links."


let private makeEmbed (client : IDiscordClient) =
  let embed = EmbedBuilder()
  embed.Title <- "ComputerCraft FAQ Bot"
  embed.Color <- Color(0x00e6e6u)
  embed.Url <- "https://github.com/SquidDev-CC/FAQBot-CC"
  embed.ThumbnailUrl <- client.CurrentUser.GetAvatarUrl()
  embed.Description <-
    "A Discord bot for answering frequently asked questions regarding CC. Please contribute and expand the list of answers on [GitHub](https://github.com/SquidDev-CC/FAQBot-CC)!"

  let addField name value =
    embed.AddField (fun x ->
      x.Name <- name
      x.Value <- value
      x.IsInline <- true)
    |> ignore

  addField
    ":information_source: **Commands**"
    "Please use the `%help` to list all possible commands.\nUse `/faq <search>` to find faqs related to your search."
  addField ":hash: **Developers**" "**HydroNitrogen** as creator and other contributors mentioned on GitHub."
  addField ":asterisk: **FAQs**" $"Currently there are {Faq.getAll () |> List.length} FAQs loaded into memory."

  let started = Diagnostics.Process.GetCurrentProcess().StartTime
  let uptime = (DateTime.Now - started).ToString(@"d' day(s), 'hh\:mm\:ss")
  let started = started.ToString(@"yyyy\-MM\-dd HH\:mm\:ss' UTC'")
  addField ":up: **Uptime information**" $"Bot started: `{started}`\nBot uptime: `{uptime}`"

  embed.Build()


let private run (context : IDiscordContext) = context.Respond(embed = makeEmbed context.Client)


type AboutTextCommand() =
  inherit ModuleBase<ICommandContext>()

  [<Command("about")>]
  [<Summary(description)>]
  member this.About() : Task = run this.Context.DiscordContext


type AboutInteractionCommand() =
  inherit InteractionModuleBase<IInteractionContext>()

  [<SlashCommand("ccfaq", description)>]
  member this.About() : Task = run this.Context.DiscordContext
