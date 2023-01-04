module FAQBotCC.Telemetry

open System.Diagnostics
open System.Diagnostics.Metrics

open OpenTelemetry
open OpenTelemetry.Trace
open OpenTelemetry.Metrics
open OpenTelemetry.Resources

let activitySource = new ActivitySource("FAQBotCC")
let metricsSource = new Meter("FAQBotCC")

let private makeResources () = ResourceBuilder.CreateDefault().AddService("faq-cc")

let makeTracerProvider (config : Config) =
  let builder =
    Sdk
      .CreateTracerProviderBuilder()
      .SetSampler(new AlwaysOnSampler())
      .SetResourceBuilder(makeResources ())
      .AddSource(activitySource.Name)
      .AddHttpClientInstrumentation()

  if config.HasMetrics then
    builder.AddOtlpExporter() |> ignore
  else
    builder.AddConsoleExporter() |> ignore

  builder.Build()


let makeMetricsProvider (config : Config) =
  let provider =
    Sdk
      .CreateMeterProviderBuilder()
      .SetResourceBuilder(makeResources ())
      .AddMeter(metricsSource.Name)
      .AddHttpClientInstrumentation()

  if config.HasMetrics then
    provider.AddOtlpExporter() |> ignore
  else
    provider.AddConsoleExporter() |> ignore

  // A little useless, but the prometheus exporter fails if there's no metrics. We should
  // probably swap to otel at some point.
  metricsSource.CreateObservableGauge(
    name = "process_up",
    description = "Is faq-cc running",
    observeValue = fun () -> 1
  )
  |> ignore

  provider.Build()
