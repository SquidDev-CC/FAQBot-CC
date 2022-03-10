FROM mcr.microsoft.com/dotnet/sdk:6.0 as builder
RUN mkdir /src
WORKDIR /src
COPY . /src/
RUN ["dotnet", "publish", "-c", "Release", "-r", "linux-x64"]

FROM scratch as artifacts
COPY /faqs /faqs
COPY --from=builder /src/bin/Release/net6.0/linux-x64/publish/FAQBot-CC /FAQBot-CC
