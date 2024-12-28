# Jun Takami - A Journalling Tool

Welcome to Jun Takami, or `jt` for short. Jun Takami is
a tool to make it easier to work with a basic Markdown journal.
Its primary features are around managing a daily log of activity
along with tracking state changes to TODO items, and keeping
or removing blocks when work is complete.

## A trivial example

You can create a new journal by running `jt init` - by default
that will happen in your home directory.

```scenario
given a jt binary on the path
given a unique home directory
when I run jt init
then a journal exists at ~/journal
```
