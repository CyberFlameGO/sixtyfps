// Copyright © SixtyFPS GmbH <info@slint-ui.com>
// SPDX-License-Identifier: GPL-3.0-only OR LicenseRef-Slint-commercial

/**
Test that the tokenizer properly rejects tokens with spaces.

This should work:

```
mod x {
    use slint::*;
    slint!{ Hello := Rectangle { } }
}
```

But his not:

```compile_fail
mod x {
    use slint::*;
    slint!{ Hello : = Rectangle { } }
}
```

*/
#[cfg(doctest)]
const basic: u32 = 0;
