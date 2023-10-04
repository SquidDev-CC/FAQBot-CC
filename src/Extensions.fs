// Terrible mish-mash of extension methods and helpers.
namespace FAQBotCC

open System
open System.Collections.Generic
open System.Threading
open System.Threading.Tasks

open System.Diagnostics
open System.Diagnostics.Metrics

open System.Runtime.CompilerServices

open Discord
open Discord.Commands


/// <summary>
/// A "handle" over an acquired lock, which is released when .Dispose() is called.
///
/// This allows you to bind the lock handle with use or use!, ensuring it is
/// released when it goes out of scope.
/// </summary>
type LockHandle private (lock : SemaphoreSlim) =
  struct
    interface IDisposable with
      member _.Dispose() = lock.Release() |> ignore
  end

  static member Wait(lock : SemaphoreSlim) =
    lock.Wait()
    new LockHandle(lock)

  static member WaitAsync(lock : SemaphoreSlim) =
    task {
      let! () = lock.WaitAsync().ConfigureAwait(false)
      return new LockHandle(lock)
    }


/// <summary>A KV pair provided as a tag for a metric.</summary>
type private Tag = KeyValuePair<string, obj>


/// <summary>
/// A helper object used in combination with <c>Histogram.Timed</c> to measure
/// how long a block of code takes.
/// </summary>
type HistogramTimer2 internal (meter : Histogram<int64>, start : int64, tag1 : Tag, tag2 : Tag) =
  struct
    interface IDisposable with
      member _.Dispose() = meter.Record(Environment.TickCount64 - start, tag1, tag2)
  end


module private Constants =
  let noReplyMention =
    let r = new AllowedMentions()
    r.MentionRepliedUser <- false
    r


/// <summary>
/// An abstraction over the various existing discord contexts which which
/// provide a basic way to respond.
/// </summary>
type IDiscordContext =
  abstract Client : IDiscordClient
  abstract User : IUser
  abstract Guild : IGuild
  abstract Respond :
    ?text : string *
    ?isTTS : bool *
    ?embed : Embed *
    ?embeds : Embed array *
    ?file : FileAttachment *
    ?components : MessageComponent ->
      Task




/// <summary>A generic mish-mash of extension methods.</summary>
module Extensions =
  type private TextDiscordContext(context : ICommandContext) =
    interface IDiscordContext with
      member _.Client = context.Client
      member _.User = context.User
      member _.Guild = context.Guild

      member _.Respond
        (
          ?text : string,
          ?isTTS : bool,
          ?embed : Embed,
          ?embeds : Embed [],
          ?file : FileAttachment,
          ?components : MessageComponent
        ) : Task =
        match file with
        | None ->
          context.Message.ReplyAsync(
            text = defaultArg text null,
            isTTS = defaultArg isTTS false,
            embed = defaultArg embed null,
            embeds = defaultArg embeds null,
            allowedMentions = Constants.noReplyMention,
            components = defaultArg components null
          )
        | Some file ->
          context.Channel.SendFileAsync(
            file,
            text = defaultArg text null,
            isTTS = defaultArg isTTS false,
            embed = defaultArg embed null,
            embeds = defaultArg embeds null,
            messageReference = MessageReference(context.Message.Id),
            allowedMentions = Constants.noReplyMention,
            components = defaultArg components null
          )


  type private InteractionDiscordContext(context : IInteractionContext) =
    interface IDiscordContext with
      member _.Client = context.Client
      member _.User = context.User
      member _.Guild = context.Guild

      member _.Respond
        (
          ?text : string,
          ?isTTS : bool,
          ?embed : Embed,
          ?embeds : Embed [],
          ?file : FileAttachment,
          ?components : MessageComponent
        ) : Task =
        match file with
        | None ->
          context.Interaction.RespondAsync(
            text = defaultArg text null,
            isTTS = defaultArg isTTS false,
            embed = defaultArg embed null,
            embeds = defaultArg embeds null,
            allowedMentions = Constants.noReplyMention,
            components = defaultArg components null
          )
        | Some file ->
          context.Interaction.RespondWithFileAsync(
            file,
            text = defaultArg text null,
            isTTS = defaultArg isTTS false,
            embed = defaultArg embed null,
            embeds = defaultArg embeds null,
            allowedMentions = Constants.noReplyMention,
            components = defaultArg components null
          )


  type IReadOnlyDictionary<'k, 'v> with
    member this.TryFindValue key =
      let (found, result) = this.TryGetValue(key)
      if found then ValueSome result else ValueNone

    member this.TryFind key =
      let (found, result) = this.TryGetValue(key)
      if found then Some result else None


  type Activity with
    member this.SetTag' key value = if this <> null then this.SetTag(key, value) |> ignore

    member this.SetOk ok =
      if this.Status = ActivityStatusCode.Unset then
        this.SetStatus(if ok then ActivityStatusCode.Ok else ActivityStatusCode.Error)
        |> ignore


  type IDiscordClient with
    member this.GetMessageAsync(reference : MessageReference) =
      task {
        match! this.GetChannelAsync(reference.ChannelId, mode = CacheMode.AllowDownload) with
        | :? IMessageChannel as channel ->
          let! message = channel.GetMessageAsync(reference.MessageId.Value, mode = CacheMode.AllowDownload)
          return if message = null then None else Some message
        | _ -> return None
      }

    member this.GetReferencedMessageAsync(message : IUserMessage) : Task<Option<IMessage>> =
      task {
        if message.Reference = null then
          return None
        else if message.ReferencedMessage <> null then
          return Some message.ReferencedMessage
        else
          return! this.GetMessageAsync(message.Reference)
      }


  type ICommandContext with
    member this.DiscordContext : IDiscordContext = TextDiscordContext(this)


  type IInteractionContext with
    member this.DiscordContext : IDiscordContext = InteractionDiscordContext(this)


  /// <summary>Create a new metrics tag.</summary>
  let tag name value : KeyValuePair<string, obj> = KeyValuePair(name, value)

  let (|IGuildUser|_|) (user : IUser) =
    match user with
    | :? IGuildUser as user -> Some user
    | _ -> None

  let (|IGuildChannel|_|) (channel : IMessageChannel) =
    match channel with
    | :? IGuildChannel as channel -> Some channel
    | _ -> None

[<Extension>]
type Extensions =
  /// <summary>Time how long a block of code takes with <c>use _ = histogram.Timed(x, y)</c></summary>
  [<Extension>]
  static member Timed(this : Histogram<int64>, tag1 : Tag, tag2 : Tag) =
    new HistogramTimer2(this, Environment.TickCount64, tag1, tag2)
