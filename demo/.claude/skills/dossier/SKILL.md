---
name: dossier
description: Build a comprehensive dossier on a person of interest using all available sources.
argument-hint: "[person name]"
---

Build a comprehensive dossier on **$ARGUMENTS** using all available sources.

## Process

1. Search the notebook for any existing notes on the subject (grep name, aliases, known associates)
2. Pull from connected services:
   - LA County property records (real estate holdings, transfers)
   - LADWP employment records
   - LA Times archives (mentions, quotes, coverage)
   - City council minutes (testimony, petitions, political connections)
   - Phone directory / reverse lookup
3. Cross-reference the subject against people and entities already in the case file - flag any connections
4. Compile into a single `ai/` note

## Output Format

Use the standard AI note header (`# Title`, `date:`, `description:`, `tags:`). Tag with `#ai-generated` and `#dossier`. Link to the subject's existing note in the description. Structure:

- **Basic information** - full name, known addresses, occupation
- **Public record** - property, employment, legal filings
- **Press coverage** - LA Times mentions, quotes, context
- **Known associates** - link to existing notes where possible
- **Red flags** - anything inconsistent, suspicious, or worth investigating
- **Assessment** - one paragraph synthesis: who is this person in the context of the current case?

## Notes

- Stick to what's in the public record. Flag speculation clearly.
- If the subject connects to an existing thread in the case, call it out explicitly with links.
- If a prior dossier exists on this person, update it rather than creating a duplicate.
