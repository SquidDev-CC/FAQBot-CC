/// <summary>
/// Utilities for looking up a string in a dictionary. This either attempts to
/// find an exact match or an approximate match with some sufficient confidence
/// interval.
///
/// This is currently only used by the doc/source lookup, but could potentially
/// be useful for FAQs too.
/// </summary>
module FAQBotCC.Lookup

open System.Collections.Generic
open FAQBotCC.Extensions

open FuzzySharp
open FuzzySharp.SimilarityRatio
open FuzzySharp.SimilarityRatio.Scorer
open FuzzySharp.SimilarityRatio.Scorer.StrategySensitive

type LookupResult<'a> =
  | Missing
  | Exact of 'a
  | Fuzzy of string * 'a

let private getBest (lookup : IReadOnlyDictionary<string, _>) : Extractor.ExtractedResult<_> list -> _ =
  function
  | [ x ] -> Fuzzy(x.Value, lookup[x.Value])
  | (x :: y :: _) when x.Score >= y.Score + 5 -> Fuzzy(x.Value, lookup[x.Value])
  // TODO: It might be interesting here to attempt to drop stems, but the fuzzy finder is probably good enough for now.
  | _ -> Missing

let findWith<'T, 'V when 'T :> IRatioScorer and 'T : (new : unit -> 'T)>
  (lookup : IReadOnlyDictionary<string, 'V>)
  query
  =
  match lookup.TryFindValue query with
  | ValueSome v -> Exact v
  | ValueNone ->
    FuzzySharp.Process.ExtractTop(
      query = query,
      choices = lookup.Keys,
      limit = 2,
      cutoff = 80,
      scorer = ScorerCache.Get<'T>()
    )
    |> List.ofSeq
    |> getBest lookup

let find (lookup : IReadOnlyDictionary<string, _>) query = findWith<DefaultRatioScorer, _> lookup query
