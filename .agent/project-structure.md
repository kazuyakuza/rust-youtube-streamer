# Project Structure

# Folders in src/

- app/ - application orchestration: startup, task coordination, error handling, shutdown
- config/ - loading, validation and exposure of the application configuration
- youtube/ - YouTube API integration: OAuth auth, broadcast lifecycle, streams, chat transport
- chat/ - chat message model and bounded visible-message queue (ChatStore)
- renderer/ - converts RenderInput to raw RGB24 video frames; no YouTube/FFmpeg dependency
- streaming/ - FFmpeg process supervision and raw-frame input pipeline

# Other folders

- .agent/ - agent context: project-info/, todos/, rules/workflow indexes and the structure map
- .kilo/ - Kilo Code integration: agents/, rules/, commands/ and plans/
- .opencode/ - opencode integration: agents/, commands/ and opencode.json
- config/ - runtime JSON configuration files
- credentials/ - OAuth credentials and token storage location (no files committed here)
- docs/ - Documentation files
- fonts/ - font files used by the renderer
- logs/ - application/chat log output directory (contents ignored; only .gitkeep placeholder versioned)
