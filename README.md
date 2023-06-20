# emdiro - lEt Me Do It foR yOu

A desktop automation tool build upon `ydotool`, `grim` and `slurp`.

## Setup

Make sure that you have permission to run `ydotoold -P 0660` and that you then have access to the created file.

The easiest way to achieve that is by setting the setuid bit on ydotoold: `sudo chmod u+s $(which ydotoold)`
