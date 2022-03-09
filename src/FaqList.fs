namespace FAQBotCC.Faqs

open System.IO
open System.Text.RegularExpressions

open Discord

open YamlDotNet.Serialization
open YamlDotNet.Serialization.NamingConventions

/// <summary>Information about an FAQ encoded in its frontmatter</summary>
type FaqInfo() =
  member val Title = "" with get, set
  member val Search = "" with get, set

/// <summary>
type Faq =
  { /// <summary>The unique name/id of the FAQ. Equal to the filename minus extension.</summary>
    Name : string

    /// <summary>The actual FAQ's contents.</summary>
    Contents : string

    /// <summary>The title of the FAQ.</summary>
    Title : string

    /// <summary>A string used to find the FAQ when using the legacy %faq command.</summary>
    Search : string }

module Faq =
  let toEmbed (faq : Faq) =
    EmbedBuilder()
      .WithTitle(faq.Title)
      .WithDescription(faq.Contents)
      .WithColor(Color(0x00e6e6u))
      .Build()

  let private yaml =
    (DeserializerBuilder())
      .WithNamingConvention(UnderscoredNamingConvention.Instance)
      .Build()

  let private frontmatter =
    new Regex("---\n(.*?)\n---\n(.*)$", RegexOptions.Multiline ||| RegexOptions.Singleline)

  let private loadFile name =
    let contents = File.ReadAllText(name)
    let matches = frontmatter.Match(contents).Groups
    let info = yaml.Deserialize<FaqInfo>(matches[1].Value)
    { Name = Path.GetFileNameWithoutExtension(name)
      Contents = matches[ 2 ].Value.Trim()
      Title = info.Title
      Search = info.Search }

  let private faqs =
    lazy
      (Directory.EnumerateFiles("faqs", "*.md")
       |> Seq.filter (fun x -> Path.GetFileName(x) <> "example.md")
       |> Seq.map loadFile
       |> List.ofSeq)

  /// <summary>
  /// Loads FAQS from the ./faq/ directory.
  /// </summary>
  let getAll () = faqs.Force()
