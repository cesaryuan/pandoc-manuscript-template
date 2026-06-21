---
name: manuscript-review
description: "提升以 Pandoc Markdown 编写的学术论文初稿时，应使用此技能。它会根据学术写作规范检查结构、写作风格和格式，并给出具体、可执行的修改建议。不用与根据审稿意见或者其他具体的意见对论文进行修改。"
--- 

# 稿件审阅

审阅当前稿件文件中的学术写作质量问题。先通读稿件，再按照下列规范的优先级依次检查，最后给出带位置标注的具体问题和修改建议。

## 审阅流程

1. 阅读完整稿件文件（`manuscript.md` 或用户指定的文件）
2. 按照下方优先级顺序逐项检查各条规范
3. 按优先级分组报告发现的问题，并附上文件位置和具体修改建议
4. 如果用户同意，再应用这些修改

## 优先级 1：关键结构问题

这些问题会影响整篇文稿，应优先处理。

**1.1 引言与相关工作分离**
- 引言和相关工作必须合并为一个统一的 Introduction
- 建议结构：问题/研究动机 → 传统方法 → 带引用的现代方法 → 研究空白 → 本文贡献 → 论文组织

**1.2 引言内部出现小节**
- Introduction 必须是连贯叙述，不能包含任何 `##` 标题
- 删除所有子小节，并重组为自然衔接的段落

**1.3 过多使用 `###`（三级标题）**
- 将其改写为带过渡语的叙述性段落
- 可使用“First, ...”“Subsequently, ...”“Building upon this, ...”等表达连接主题

## 优先级 2：主要行文流畅性问题

在结构正确之后再处理这些问题。

**2.1 伪标题**：将加粗文本当作标题使用

识别模式：段首出现 `**Label**: content...`。应改写为自然叙述。

修改前：
```markdown
**Data Collection**: We collected data from multiple sources...

**Data Cleaning**: The data was preprocessed by removing outliers...
```

修改后：
```markdown
Data was collected from multiple sources including sensor networks and historical
records. The raw data underwent preprocessing to remove outliers and handle missing
values through interpolation.
```

**2.2 列表过多的写法**：用简短项目符号代替段落

应使用枚举式衔接语将其改写为叙述性段落。

修改前：
```markdown
The procedure includes:
- Data preprocessing
- Feature extraction
- Model training
```

修改后：
```markdown
The procedure consists of three main stages. First, data preprocessing cleans
and normalizes the input. Second, feature extraction captures discriminative
characteristics. Third, model training optimizes the objective function.
```

**2.3 重复的图表引导句式**

识别：段落以 `@fig:label shows/presents/illustrates...` 开头。

修改方式：先写叙述内容，再在句末或段末放置图引用。
```markdown
The experimental results demonstrate superior performance, with the proposed
method achieving 95% accuracy, as shown in @fig:results.
```

表达可适当变化，例如“as shown in”“depicted in”“presented in”，或使用括号形式 `(@fig:label)`。

## 优先级 3：次要格式问题

在结构和行文流畅性修正完成后再进行润色。

**3.1 过度使用加粗**
- 允许：表格中的最优结果、贡献陈述中的作者名称
- 不允许：正文强调、列表项标签、伪标题

**3.2 交叉引用语法错误**
- 检查所有 `@fig:`、`@tbl:`、`@eq:`、`@sec:` 引用是否都指向有效标签

**3.3 引文格式**
- 括号式引用：`[@key]` 或 `[@key1; @key2]`
- 叙述式引用：`@key showed that...`

## 附加检查项

**段落质量：**
- 每段应有 3-7 句，围绕一个中心思想展开，并以主题句开头
- 标记单句段落（过渡段除外）
- 标记过长段落（超过 10 句）

**章节结构：**
- Introduction：不设子小节，并整合相关工作
- Methods/Results/Discussion：允许子小节，但每节最多建议 3-5 个
- Conclusion：通常不设子小节

**编号列表：**
- 每一项都应是完整句子或完整段落，而不是简短词组
