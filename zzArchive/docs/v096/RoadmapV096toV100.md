# We need an updated roadmap for our current state

## v0.9.6

- [ ] Our single parseltongue binary is NOT fixed, not working, we need to get them working
- [ ] Our we need to update @parseltongue-install-v092.sh to the right level
- [ ] We need a proper checklist for our release
    - [ ] How to name the release
    - [ ] How to tag the release
    - [ ] How to build the release
    - [ ] How to do a command list check for our pareto commands in a temp folder
    - [ ] How to publish the release
    - [ ] How to do a post release check if the published release is working
- [ ] We need a new PRD based on what the tool does and the agent is saying
- [ ] We need to eliminate test interfaces from our ingestion pipeline - they pollute our context, so remove them from command options and everywhere also - just make sure for each command we run we show the message tests are intentionally exlcluded from these analysis
- [ ] We need to gracefully manage the half implemented crates
- [ ] We need to update the ReadME thoroughly with what exists by doing real checks on our codebase itself

# v 2.0.0
- [ ] Claude code will need an edit interface
- [ ] We need a new agentic tool to work on backend code which does not need frontend stuff, and can be validated purely by CLI
     - [ ] DevOps
     - [ ] System Programming
     