Seraphina Studio (Windows x64)

Double-click SeraphinaStudio.exe. Keep the assets folder and out.* files next to it.
You can also open a file directly:  SeraphinaStudio.exe path\to\image.psd
Built with the x86_64-pc-windows-gnu toolchain. Tested under Wine 10 on Linux
(a proxy, not real Windows) and built + unit-tested by CI on windows-latest.

RECORDING AND STREAMING NEED FFMPEG (not bundled)
-------------------------------------------------
Record (MP4: H.264 + AAC) and Go live (RTMP) run ffmpeg as a subprocess.
ffmpeg is not included because a static GPL build (needed for libx264) is
about 170 MB. To enable it:
  1. Download a Windows build, e.g.
       https://www.gyan.dev/ffmpeg/builds/          (ffmpeg-release-essentials.zip)
       https://github.com/BtbN/FFmpeg-Builds/releases  (ffmpeg-master-latest-win64-gpl.zip)
  2. Copy bin\ffmpeg.exe (and ffprobe.exe if you like) next to SeraphinaStudio.exe,
     or add its folder to PATH, or set SERAPHINA_FFMPEG=C:\full\path\ffmpeg.exe
  3. Restart Seraphina Studio. Stream mode > Output shows "ffmpeg found".
Lookup order: SERAPHINA_FFMPEG, next to the exe (or .\ffmpeg\bin\), then PATH.
Without ffmpeg, Record / Go live show a clear "ffmpeg was not found" message.

ffmpeg licence: those builds are GPL v3 (they include libx264). If you
redistribute Seraphina Studio together with ffmpeg.exe you must ship the
ffmpeg LICENSE and offer its source (https://ffmpeg.org/download.html).
Seraphina Studio only runs ffmpeg as a separate program; it does not link it.

Output settings (Settings > Output): RTMP URL, stream key, size, fps,
bitrate, audio (none / silent AAC / DirectShow device), record folder,
reconnect delay and attempts. Settings are saved in
%USERPROFILE%\.seraphina\studio\output.json; the stream key is stored on its
own in %USERPROFILE%\.seraphina\studio\stream_key, shown masked, and never
sent to the AI assistant or written to logs. Recordings default to
%USERPROFILE%\.seraphina\studio\recordings.

AI (optional). Set these in your environment before launching:
  XAI_API_KEY             xAI key for the Seraphina assistant and Grok Imagine (or SERAPHINA_GROK_API_KEY)
  SERAPHINA_MODEL         chat model (default grok-4.7)
  SERAPHINA_BASE_URL      OpenAI-compatible endpoint (default https://api.x.ai/v1)
  SERAPHINA_VIDEO_MODEL   default grok-imagine-video-1.5 (billed per second)
  SERAPHINA_IMAGE_MODEL   default grok-imagine-image-2.0
Without a key the assistant runs offline and says so; nothing is faked.
The Seraphina.AGI CLI (remember / recall / triad) is used when `seraphina`
is on PATH (pip install seraphina-agi).
Set SERAPHINA_NATIVE_FRAME=1 to use the normal Windows title bar.
