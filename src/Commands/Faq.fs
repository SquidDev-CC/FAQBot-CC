/// <summary>Provides commands to read FAQs</summary>
module FAQBotCC.Commands.Faq

open System
open System.Threading.Tasks

open Discord
open Discord.Interactions
open Discord.Commands

open FAQBotCC.Faqs
open FAQBotCC.Extensions


/// <summary>
/// Provides an %faq (or %f, %info, %i) command. This searches for a FAQ based
/// on a user-provided regex.
/// </summary>
type FaqTextCommand() =
  inherit ModuleBase<ICommandContext>()

  [<Command("faq")>]
  [<Alias("f", "info", "i")>]
  [<Summary("Retrieves FAQs related to given keyword(s).")>]
  member this.Faq([<Remainder>] query : string) : Task =
    let context = this.Context.DiscordContext
    let matching =
      Faq.getAll ()
      |> List.filter (fun x -> x.Search.Contains(query, StringComparison.InvariantCultureIgnoreCase))
      |> List.map Faq.toEmbed
    match matching with
    | [] ->
      context.Respond(
        "Sorry, I did not find any faqs related to your search. Please contribute to expand my faq list <https://github.com/SquidDev-CC/FAQBot-CC> or use `/faq`."
      )
    | [ faq ] -> context.Respond(embed = faq)
    | faqs -> context.Respond(text = "Multiple matching FAQs. Seriously, just use `/faq`.", embeds = Array.ofList faqs)


/// <summary>
/// Generate /faq sub-commands for each loaded FAQ.
/// </summary>
let getSlashCommands (builder : Interactions.Builders.ModuleBuilder) =
  builder.WithGroupName("faq").WithDescription("Find an FAQ.")
  |> ignore

  for faq in Faq.getAll () do
    let embed = Faq.toEmbed faq
    builder.AddSlashCommand(
      faq.Name.Replace(".", ""),
      ExecuteCallback(fun context _ _ _ -> context.DiscordContext.Respond(embed = embed)),
      fun (builder : Builders.SlashCommandBuilder) -> builder.WithDescription(faq.Title) |> ignore
    )
    |> ignore
