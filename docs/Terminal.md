# Terminal Support

This document describes extentions to [Basic Text] adding ANSI-style terminal
features. Is is experimental.

[Basic Text]: BasicText.md

## Additional Input Options

The following options are added to the options for Basic Text:

| Name              | Type    | Applicability               |
| ----------------- | ------- | --------------------------- |
| Immediate input   | Boolean | Input streams               |
| Hidden input      | Boolean | Input streams               |

In Immediate input mode, treat each keypress as if it were followed by a
newline, though the extra newline is not sent to the application. And as a
special case, U+C (FF) is not replaced in immediate input mode.

In Hidden input mode, terminal implementations should not echo input
characters back to the terminal.

## Output Feature Sets

Terminal output features are grouped into sets, which can be supported
independently or in combination:

 - [Line-oriented output](#line-oriented-output), eg. for `readline`
 - [Full-screen output](#full-screen-output), eg. for `vim`
 - [Color](#color), eg. for color `ls`
 - [Custom Title](#custom-title), eg. for shell command prompts

#### Line-oriented output

This feature adds line-oriented editing features.

The following control codes are recognized:

| Code   | Meaning                                                     |
| ------ | ----------------------------------------------------------- |
| U+7    | [Alert](#alert)                                             |
| U+8    | [Move cursor back one column](#move-cursor-back-one-column) |
| U+9    | Horizontal Tab                                              |
| U+A    | [End of line](#end-of-line)                                 |
| U+C    | [FF Terminal Compatibility](#ff-terminal-compatibility)     |
| U+D    | [Carriage Return](#carriage-return)                         |
| U+7F   | [No Effect](#no-effect)                                     |

The following escape sequences are recognized:

| Sequence             | Meaning                      | Notes |
| -------------------- | ---------------------------- | ----- |
| `␛[0K`               | Clear to the end of the line | The `0` is optional |
| `␛[2K`               | Clear the entire line        |       |

##### Alert

U+7 (BEL) on output to a terminal may produce an acoustic indication, a
visual indication, or do nothing.

##### Move cursor back one column

U+8 (BS) on output to a terminal moves the cursor back one column, but not
past the first column.

##### End of line

U+A on output to a terminal moves the cursor to the beginning of the next
line, scrolling the output if needed.

##### FF Terminal Compatibility

U+C on output to a terminal moves the cursor to the next line without
changing the column.

##### Carriage Return

U+D (CR) on output to a terminal moves the cursor to the first column of the
current line.

##### No Effect

U+7F (DEL) on output to a terminal has no effect.

#### Full-screen output

This feature set adds a "full screen" mode which may be enabled at runtime,
which supports two-dimensional cursor positioning, scrolling, screen clearing,
and related features.

In the default mode, the following escape sequences are recognized:

| Sequence             | Meaning                  | Notes |
| -------------------- | ------------------------ | ----- |
| `␛[?1049h`           | Enter full-screen mode, with a clear screen and default settings | |

The following escape sequences are recognized within full-screen mode:

| Sequence             | Meaning                  | Notes |
| -------------------- | ------------------------ | ----- |
| `␛7`                 | `save_cursor`            | TODO: Do we need this? |
| `␛8`                 | `restore_cursor`         | Ditto |
| `␛H`                 | `set_tab`                | Ditto |
| `␛M`                 | `scroll_reverse`         | TODO: Is this different on Windows? |
| `␛[«n»@`             | `parm_ich(«n»)`          | `«n»` may be omitted and defaults to 1 |
| `␛[«n»A`             | `parm_up_cursor(«n»)`    | Ditto |
| `␛[«n»B`             | `parm_down_cursor(«n»)`  | Ditto |
| `␛[«n»C`             | `parm_right_cursor(«n»)` | Ditto |
| `␛[«n»D`             | `parm_left_cursor(«n»)`  | Ditto |
| `␛[«n»G`             | `column_address(«n»)`    | Ditto |
| `␛[«line»;«column»H` | `cursor_address(«row», «column»)` | `«line»;«column»` may be omitted and default to `1;1` |
| `␛[«n»I`             | `tab(«n»)`               | `«n»` may be omitted and defaults to 1 |
| `␛[0J`               | `clr_eos`                | The `0` is optional |
| `␛[1J`               | Clear the screen from the beginning to the current cursor position | |
| `␛[2J`               | Clear the screen         | Unlike `clear_screen`, this doesn't change the cursor position |
| `␛[«n»L`             | `insert_line(«n»)`       | `«n»` may be omitted and defaults to 1 |
| `␛[«n»M`             | `parm_delete_line(«n»)`  | Ditto |
| `␛[«n»P`             | `parm_dch(«n»)`          | Ditto |
| `␛[«n»S`             | `parm_index(«n»)`        | Ditto |
| `␛[«n»T`             | `parm_rindex(«n»)`       | Ditto |
| `␛[«n»X`             | `erase_chars(«n»)`       | Ditto |
| `␛[«n»Z`             | `cbt(«n»)`               | Ditto |
| `␛[«n»d`             | `row_address(«n»)`       | Ditto |
| `␛[«line»;«column»f` | Same as the similar sequence ending in `H` | |
| `␛[3g`               | `clear_all_tabs`         | TODO: do we need this? |
| `␛[?25h`             | `cursor_visible`         |       |
| `␛[?1049h`           | Clear the screen and reset full-screen settings to defaults |
| `␛[?2004h`           | Enable bracketed paste mode |    |
| `␛[?25l`             | `cursor_invisible`       |       |
| `␛[?1049l`           | Exit full-screen mode and restore the terminal to its prior state | |
| `␛[?2004l`           | Disable bracketed paste mode |   |
| `␛[!p`               | Reset the terminal to default settings, without clearing the screen | |
| `␛[«top»;«bottom»r`  | `change_scroll_region(«top», «bottom»)` | `«top»;«bottom»` may be omitted and default to `1;«viewpoint-height»` |

TODO: Describe the behavior on on the rightmost column and bottom-most line,
and other traditionally underspecified things.

TODO: Describe parameters in more detail, including the syntax for numeric
and string parameters, and min/max valid values for numeric parameters.

#### Color

This feature set adds color and display attributes such as bold, underline,
and italics.

This feature defines the following escape sequences on output:

| Sequence              | Meaning                  | Notes |
| --------------------- | ------------------------ | ----- |
| `␛[…m`                | `set_attributes(…)`      | Set text attributes; see below for the meaning of `…` |
| `␛[38;2;«r»;«g»;«b»m` | Set foreground color to RGB `«r»`, `«g»`, `«b»` | Values are from 0-255 |
| `␛[48;2;«r»;«g»;«b»m` | Set background color to RGB `«r»`, `«g»`, `«b»` | Ditto |

In the `…` form above, the `…` may be replaced by up to 16 `;`-separated
sequences from the following:

| Sequence | Meaning                   | Notes |
| -------- | ------------------------- | ----- |
| `0`      | Normal (default)          |       |
| `1`      | Bold                      |       |
| `2`      | Faint                     | Faint may not appear visually distinct on some platforms |
| `4`      | Underlined                | May be "simulated with color". Applications may wish to use U+332 instead. |
| `7`      | Inverse                   |       |
| `22`     | Not bold or faint         |       |
| `23`     | Not italicized            |       |
| `24`     | Not underlined (any kind) |       |
| `27`     | Not inverse               |       |
| `29`     | Not crossed-out           |       |
| `30`     | Foreground Black          |       |
| `31`     | Foreground Red            |       |
| `32`     | Foreground Green          |       |
| `33`     | Foreground Yellow         | May appear brown on some platforms |
| `34`     | Foreground Blue           |       |
| `35`     | Foreground Magenta        |       |
| `36`     | Foreground Cyan           |       |
| `37`     | Foreground White          |       |
| `39`     | Foreground default        |       |
| `40`     | Background Black          |       |
| `41`     | Background Red            |       |
| `42`     | Background Green          |       |
| `43`     | Background Yellow         |       |
| `44`     | Background Blue           |       |
| `45`     | Background Magenta        |       |
| `46`     | Background Cyan           |       |
| `47`     | Background White          |       |
| `49`     | Background default        |       |
| `90`     | Foreground bright Black   | Bright colors may not appear visually distinct on some platforms |
| `91`     | Foreground bright Red     |       |
| `92`     | Foreground bright Green   |       |
| `93`     | Foreground bright Yellow  |       |
| `94`     | Foreground bright Blue    |       |
| `95`     | Foreground bright Magenta |       |
| `96`     | Foreground bright Cyan    |       |
| `97`     | Foreground bright White   |       |
| `100`    | Background bright Black   |       |
| `101`    | Background bright Red     |       |
| `102`    | Background bright Green   |       |
| `103`    | Background bright Yellow  |       |
| `104`    | Background bright Blue    |       |
| `105`    | Background bright Magenta |       |
| `106`    | Background bright Cyan    |       |
| `107`    | Background bright White   |       |

Not all terminal support all colors; when a requested color is unavailable,
terminals may substitute the closest available color.

#### Custom Title

This feature set adds the ability to set a custom window title.

This feature defines the following escape sequences on output:

| Sequence              | Meaning                  | Notes |
| --------------------- | ------------------------ | ----- |
| `␛]0;«string»␇`       | Sets the terminal's title to `«string»` | Implementations may implicitly add a prefix and/or truncate the string |
| `␛]2;«string»␇`       | Sets the terminal's title to `«string»` | Ditto |

### Binary

Arbitrary bytes are permitted, without translation.

## Terminal input

Most keys have obvious mappings to Unicode scalar value sequences. This section
describes mapping for special keys read from a terminal. Note that depending on
the stream [class](#the-classes), some of these sequences may be replaced by
replacement sequences.

Three modifiers are recognized: Ctrl, Alt, and Shift. In environments with Meta
keys, Meta is mapped to Alt.

Input key sequences are at most 8 bytes long.

### Terminal input control codes

The following [control codes](#control-code) are recognized:

| Code   | Meaning     | Notes                                               |
| ------ | ----------- | --------------------------------------------------- |
| U+0 | Ctrl-Space  |                                                     |
| U+8 | Ctrl-`H`    | Despite U+8 being historically called "backspace" in ACSII, this isn't the backspace key |
| U+9 | Tab         |                                                     |
| U+A | Enter       | U+A means "end of line"                          |
| U+11 | Ctrl-`Q`    | When enabled in the terminal input mode             |
| U+13 | Ctrl-`S`    | When enabled in the terminal input mode             |
| U+1B | Escape      | When read in in immediate input mode                |
| U+1C | Ctrl-`\`    | When enabled in the terminal input mode             |
| U+1D | Ctrl-`]`    |                                                     |
| U+1E | Ctrl-`^`    |                                                     |
| U+1F | Ctrl-`_`    |                                                     |
| U+7F | Backspace   | This is the backspace key                           |

The following control codes are interpreted by the implementation and not
passed on to applications:

| Code   | Commonly typed as  | Behavior                                     |
| ------ | ------------------ | -------------------------------------------- |
| U+3 | Ctrl-`C`           | Terminate the program, when not enabled in the terminal input mode |
| U+9 | Tab                | No effect when modifiers include Alt         |
| U+D | Ctrl-`M`           | Send U+A to the program, when read in a single input call in immediate input mode |
| U+11 | Ctrl-`Q`           | No effect when not enabled in the terminal input mode |
| U+13 | Ctrl-`S`           | No effect when not enabled in the terminal input mode |
| U+1A | Ctrl-`Z`           | Suspend the program                          |
| U+1C | Ctrl-`\`           | Terminate the program, when not enabled in the terminal input mode |
| U+60 | `` ` ``            | No effect when modifiers include Alt         |

Except as specified otherwise above, U+1 through U+1A are recognized as
Ctrl-`A` through Ctrl-`Z`, respectively.

Codes with values U+0 through U+7F, except for U+5B (`[`) and
U+5D (`]`), may be preceeded by U+1B indicating the Alt modifier.

When a program is resumed from being suspended, any streams open in immediate
input mode are passed a U+C (Ctrl-L). Applications are encouraged to
interpret Ctrl-L as a command to redraw the screen.

### Terminal input escape sequences

The following escape sequences are recognized when they are read as a single
input call in immediate input mode:

| Sequence     | Meaning              | Notes |
| ------------ | -------------------- | ----- |
| `␛[A`        | Up                   |       |
| `␛[B`        | Down                 |       |
| `␛[C`        | Right                |       |
| `␛[D`        | Left                 |       |
| `␛[F`        | End                  |       |
| `␛[H`        | Home                 |       |
| `␛[1«m»A`    | Up                   | Same as above, but with modifiers |
| `␛[1«m»B`    | Down                 | Ditto |
| `␛[1«m»C`    | Right                | Ditto |
| `␛[1«m»D`    | Left                 | Ditto |
| `␛[Z`        | Shift-Tab            |       |
| `␛[1«m»~`    | Home                 | Same as above, but with modifiers |
| `␛[2«m?»~`   | Insert               |       |
| `␛[3«m?»~`   | Delete               |       |
| `␛[4«m»~`    | End                  | Same as above, but with modifiers |
| `␛[5«m?»~`   | Page Up              |       |
| `␛[6«m?»~`   | Page Down            |       |
| `␛[11«m?»~`  | F1                   | These use the "old xterm"/CSI values, rather than vt102/vt220/SS3/Windows values |
| `␛[12«m?»~`  | F2                   |       |
| `␛[13«m?»~`  | F3                   |       |
| `␛[14«m?»~`  | F4                   |       |
| `␛[15«m?»~`  | F5                   |       |
| `␛[17«m?»~`  | F6                   | (yes, 16 really is skipped) |
| `␛[18«m?»~`  | F7                   |       |
| `␛[19«m?»~`  | F8                   |       |
| `␛[20«m?»~`  | F9                   |       |
| `␛[21«m?»~`  | F10                  |       |
| `␛[23«m?»~`  | F11                  | (yes, 22 really is skipped) |
| `␛[24«m?»~`  | F12                  |       |
| `␛[200«m?»~` | Begin Paste          | Only emitted when bracketed paste mode is activated |
| `␛[201«m?»~` | End Paste            | Ditto |

`«m»` is a modifier sequence:

| Sequence | Shift | Alt | Ctrl |
| -------- | :---: | :-: | :--: |
| `;2`     |   ✓   |     |      |
| `;3`     |       |  ✓  |      |
| `;4`     |   ✓   |  ✓  |      |
| `;5`     |       |     |  ✓   |
| `;6`     |   ✓   |     |  ✓   |
| `;7`     |       |  ✓  |  ✓   |
| `;8`     |   ✓   |  ✓  |  ✓   |

and `«m?»` is an optional modifier sequence.

In environments with keys F13 through F24, they are mapped to F1 through F12
with the shift modifier.

As special cases, Delete, Insert, Home, End, Page Up and Down, and F1 and F12
with the Ctrl-Alt or Ctrl-Alt-Shift modifiers are reserved and not passed on
to the application.
