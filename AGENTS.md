The following is the contents of AGENTS.md file automatically injected into the system message.

## Features

`susnotes` is a local note notes / knowledge base app written in pure Rust, that is being developed by you (the AI) according to the following specification:

- Notes are stored as encrypted markdown files. Name of the file is name of the note and is displayed at the top of the note, editing it edits the name. Everything is stored in a folder specified by the user.
- All notes are encrypted and protected by a password.
- A note can link to another note via #note_name or [[note name]]. When a user opens a note and it doesn't exist, it is created automatically.
- When a note opens, it is opened in view mode, where markdown is rendered. Clicking on any content enters edit mode where you can see and edit syntax-highlighted markdown source. However, if you click and drag to select something, it will remain in view mode for easy copying.
- Code blocks with syntax highlighting, and a copy button in view mode.
- Whenever an image is pasted, it should be saved to local storage. Images should display in view mode with the ability to resize them with the mouse (in view mode).
- LaTeX support.
- Support for attachments. Unsupported files open with the default application configured for that type of file. Eventually we want to directly support various formats, such as PDFs, with additional note-taking functionality.
- Clean, modern UI. Light and dark themes.
- An AI-friendly API to interact with notes programmatically.

## Journal

Journal is a special page that opens when the susnotes is opened. When you open journal, at the top it shows a note for the current day (e.g. "2026-09-25"), automatically creating it if it doesn't exist. Below it, it shows all previous non-empty daily pages. The pages are separated by a thin vertical line and the date is displayed above the line. Depending on how user chooses to organize the notes, this may be the main page for all the content. For example, if you need to save important information, you might just open susnotes and quickly paste it to today's note. Next day, if you want to add additional information, you can either scroll down to past day and append it here, or paste to the new day. This creates a very frictionless environment for taking notes without worrying about organizing them.

## Folder structure

User can specify a portable folder which contains all notes and configs, and can be loaded on any machine with susnotes:

- assets: images and attachments
- pages: markdown files
- config: app settings

## Technology

**99+% Rust**. Dependencies should also be pure Rust whenever possible.

Note: Zed (cloned `/home/jj/distrobox/ubuntu/projects/zed`) is a code editor written entirely in Rust. Can be used for reference.

## Coding guide for AI agents

- All code in this project was written by you in previous sessions, and you have full control over all code, dependencies, tooling, etc.
- You will keep developing this app in future sessions, so ensure maintainability.
- Do not abuse `unsafe`.
- You should be very proactive with maintaining this project, if you encounter a difficulty, make sure to improve the code or the tooling so that future sessions will not encounter it. This includes improving logging, tests, development documentation, and setting up development tools. It is also completely acceptable to add and change dependencies as well do big rewrites if necessary.
- A thorough logging system should be set up so that any issues the user might have while using the app are apparent from the logs.
- You can edit everything except this file (AGENTS.md), this file is written by the user.

## Development workflow

`git` and `gh` are installed and configured. Before making any changes, you must switch to a new branch, and after you are done, make a pull request. The user will then trigger a code review workflow, and your PR will be accepted and merged, or you will receive a comment on what needs to be refined. Always follow this workflow unless explicilty asked otherwise by user.
