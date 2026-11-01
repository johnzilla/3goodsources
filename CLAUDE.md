# 3goodsources

## Skill routing

When the user's request matches an available skill, ALWAYS invoke it using the Skill
tool as your FIRST action. Do NOT answer directly, do NOT use other tools first.
The skill has specialized workflows that produce better results than ad-hoc answers.

Key routing rules:
- Save progress, checkpoint, resume → invoke checkpoint
- Code review, check my diff → invoke code-review
