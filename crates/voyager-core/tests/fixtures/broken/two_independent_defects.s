; EXPECT: UnclosedBlockComment, UnmatchedIf
IF (Y=1)
    Z = 2
X = 1
/* this comment never closes, and everything after it is swallowed
