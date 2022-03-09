/// <summary>
/// Provides a mechanism for fetching a web page and extracting data from it,
/// with built-in caching.
///
/// This caches based on both a time-to-live and the page's ETag.
/// </summary>
module FAQBotCC.CachedRequest

open System
open System.Threading
open System.Threading.Tasks
open System.Net
open System.Net.Http
open System.Net.Http.Headers

open Microsoft.Extensions.Logging

type private result<'a> = { eTag : string option; Expiry : int64; Value : 'a }

type t<'result> =
  private
    { Client : HttpClient
      Logger : ILogger
      Url : string
      Ttl : int64
      Compute : string -> 'result

      mutable Value : Option<result<'result>>
      Lock : SemaphoreSlim }


let make client logger url ttl compute : t<_> =
  { Client = client
    Logger = logger
    Url = url
    Ttl = ttl
    Compute = compute
    Value = None
    Lock = new SemaphoreSlim(1) }


let private updateCache (cache : t<'result>) (response : HttpResponseMessage) value =
  let Etag =
    if response.Headers.ETag = null then None else Some response.Headers.ETag.Tag
  cache.Value <- Some { Value = value; Expiry = Environment.TickCount64 + cache.Ttl; eTag = Etag }
  value


let private compute (cache : t<'result>) : Task<'result> =
  task {
    use! _lock = LockHandle.WaitAsync(cache.Lock)

    try
      match cache.Value with
      | Some { Value = value; Expiry = expiry } when expiry >= Environment.TickCount64 -> return value
      | _ ->
        // Make a request, using the etag where available
        use! response =
          use request = new HttpRequestMessage(HttpMethod.Get, cache.Url)
          match cache.Value with
          | Some { eTag = Some etag } -> request.Headers.IfNoneMatch.Add(EntityTagHeaderValue(etag))
          | _ -> ()
          cache.Client.SendAsync(request)

        match response.StatusCode, cache.Value with
        | HttpStatusCode.NotModified, Some { Value = value } ->
          cache.Logger.LogInformation("ETag matched, not re-computing")
          return updateCache cache response value
        | _ ->
          response.EnsureSuccessStatusCode() |> ignore
          let! content = response.Content.ReadAsStringAsync()
          return
            cache.Compute content
            |> updateCache cache response

    with
    | e when cache.Value.IsSome ->
      cache.Logger.LogError(e, "Error in getting resource")
      return (Option.get cache.Value).Value
  }


let get (cache : t<'result>) : Task<'result> =
  match cache.Value with
  | Some { Value = value; Expiry = expiry } when expiry >= Environment.TickCount64 -> Task.FromResult value
  | _ -> compute cache
