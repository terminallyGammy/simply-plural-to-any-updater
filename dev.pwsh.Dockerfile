FROM mcr.microsoft.com/dotnet/sdk:9.0

ENV POWERSHELL_TELEMETRY_OPTOUT=1

COPY ./start-vrcupdater.ps1 ./

CMD ["pwsh", "start-vrcupdater.ps1"]

