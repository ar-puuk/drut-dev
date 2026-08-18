# Getting Started

This walks through the CLI against a small sample script. If you're only using
the VS Code/Open VSX extension, the same `check`/`format` behavior happens
automatically as you type and save — skip ahead to the
[Editor Guide](editor-guide.md).

## 1. A sample script

Save this as `sample.s`:

```text
run pgm=matrix
  mati=base.mat,mo=out.mat
  if(i==1)
       ZONES = 100
  endif
endrun
```

It's structurally valid Cube Voyager — but the indentation is inconsistent (2
spaces under `RUN`, then 7 under `IF`) and `IF(i==1)` has no space before its
condition.

## 2. Check it

```powershell
drut check sample.s
```

**Expected output**: nothing, and an exit code of `0`. `check` only prints when
it finds a real structural problem (an unmatched `IF`/`LOOP`/`RUN`/`PROCESS`, an
unclosed comment, and similar) — a clean script produces no output at all, the
same way a passing test suite often does.

## 3. See what formatting would change

```powershell
drut format sample.s --diff --isolated
```

(`--isolated` skips `drut.toml` discovery for this one run, so the output below
is reproducible regardless of any config file elsewhere on your machine — see
the [Configuration Reference](configuration-reference.md) for what `--isolated`
skips.)

**Expected output**:

```diff
--- sample.s
+++ sample.s
@@ -1,6 +1,6 @@
 run pgm=matrix
-  mati=base.mat,mo=out.mat
-  if(i==1)
-       ZONES = 100
-  endif
+    mati=base.mat,mo=out.mat
+    if(i==1)
+        ZONES = 100
+    endif
 endrun
```

Every nested line is now indented to a consistent 4 spaces per level (the
built-in default — see [`indent_width`](configuration-reference.md#indent_width)
to change it). Nothing about the script's meaning changed — no line was
reordered, no keyword was invented or removed.

## 4. Write the change

```powershell
drut format sample.s --write --isolated
```

Reformats `sample.s` in place. `--write`, `--check` (report which files would
change, write nothing), and `--diff` (shown above) are mutually exclusive — see
the [CLI Reference](cli-reference.md#format) for the full flag list, including
`--control-words-casing`, `--operator-spacing`, and every other formatting axis.

## Next steps

- Set up your editor for live diagnostics/formatting: [Editor Guide](editor-guide.md).
- Share these settings with your team via `drut.toml`: [Configuration Reference](configuration-reference.md).
- Wire an AI coding assistant to Drut: [MCP Guide](mcp-guide.md).
