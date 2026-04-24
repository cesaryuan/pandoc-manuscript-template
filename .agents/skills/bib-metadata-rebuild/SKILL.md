---
name: bib-metadata-rebuild
description: 当需要系统检查或重建 `.bib` 参考文献条目的元数据时，应使用此技能，尤其适用于作者姓名格式不规范、期刊或会议信息放错字段、年份或页码可疑、以及需要优先依据 DOI 或官方 API 纠正 BibTeX 条目的情况。
---

# Bib 元数据重建

处理 `references.bib` 或用户指定的 `.bib` 文件，按“优先依据 DOI 官方记录重建条目，而不是只修补局部字段”的原则，系统检查并规范参考文献元数据。

## 适用场景

- 用户明确要求检查 `.bib` 元数据错误
- 用户指出作者姓名格式、题名、期刊、卷期页码、条目类型或字段位置不标准
- 用户要求“根据 DOI 重构”“从头整理”“调用 API 确认”
- 现有 `.bib` 大量使用 `@misc + note` 混装期刊或会议信息

## 总体原则

1. 优先保留现有 cite key，除非用户明确要求重命名
2. 对有 DOI 的条目，优先使用 DOI 内容协商或官方元数据重建，而不是手工补丁式修改
3. 对无 DOI 的条目，先按题名搜索正式 DOI；实在找不到时再保留为次优来源
4. 目标是生成“字段位置正确、类型合理、作者姓名规范、核心元数据完整”的 BibTeX，而不是只让文献“勉强能用”
5. 除软件手册等特殊条目外，尽量保证每条文献都有 `doi` 和 `url`

## 工作流程

### 1. 盘点全表

先通读整个 `.bib` 文件，识别以下情况：

- 条目总数、键名模式、是否存在重复 key
- 哪些条目已有 DOI
- 哪些条目缺 DOI 但可能是正式论文
- 哪些条目是软件、手册、预印本、会议论文、期刊论文、书章节
- 哪些条目把 `journal`、`booktitle`、`pages`、`volume`、`number` 塞进了 `note`

优先把条目分为三类：

- `有 DOI，可直接重建`
- `无 DOI，但可通过题名补查 DOI`
- `无 DOI，且应保留为手工整理条目`

### 2. 依据 DOI 抓取权威元数据

对有 DOI 的条目，优先尝试 DOI 内容协商：

```bash
curl -L -H 'Accept: application/x-bibtex; charset=utf-8' "https://doi.org/<DOI>"
```

必要时改用 Citeproc JSON：

```bash
curl -L -H 'Accept: application/citeproc+json' "https://doi.org/<DOI>"
```

可接受的数据源优先级：

1. `doi.org` 内容协商返回的 BibTeX 或 Citeproc JSON
2. Crossref API
3. 出版社落地页
4. arXiv 官方 DOI 或 arXiv API

如果 DOI 落地页不是标准 Crossref/出版社返回，而是中文中转页或多重解析页，至少提取并核对：

- 正式题名
- 作者列表
- 期刊或会议名称
- 年份
- 卷期页码
- DOI 本身

### 3. 重建条目，而不是局部修补

对每个 DOI 条目，尽量用官方返回结果重建完整条目内容，同时保留原 cite key。

常见类型建议：

- `@article`：期刊论文
- `@inproceedings`：会议论文
- `@incollection`：Springer/ECCV 这类书章节式会议论文
- `@misc`：arXiv 预印本
- `@manual`：软件手册

优先保留的核心字段：

- `author`
- `title`
- `journal`
- `booktitle`
- `publisher`
- `year`
- `volume`
- `number`
- `pages`
- `doi`
- `url`

一般应移除或谨慎保留的噪声字段：

- `note` 中被错误混装的期刊或会议信息
- `issn`
- `isbn`
- `keywords`
- `abstract`
- `copyright`
- `collection`
- `series`

### 4. 规范字段位置

重点检查并改正以下问题：

- 期刊名应在 `journal`，不要塞在 `note`
- 会议名应在 `booktitle`
- 书章节式会议论文优先用 `@incollection`
- 卷号在 `volume`，期号在 `number`
- 页码或文章号在 `pages`
- DOI 在 `doi`
- DOI 跳转地址在 `url`

如果原条目类似这样：

```bibtex
@misc{key,
  title = {...},
  year = {...},
  note = {Journal 12, 123--130},
  doi = {...}
}
```

应重构为：

```bibtex
@article{key,
  author = {...},
  title = {...},
  journal = {...},
  year = {...},
  volume = {...},
  pages = {...},
  doi = {...},
  url = {...}
}
```

### 5. 规范作者姓名

作者字段不要保留缩写式或不稳定的拼法，优先采用 DOI 官方记录中的完整姓名。

目标格式：

```bibtex
author = {Family, Given and Family, Given and Family, Given}
```

注意：

- 保留重音符号、变音符号和连字符
- `Jr.`、`del`、`van`、`de` 等姓名成分要谨慎处理
- 不要把作者列表写成逗号分隔的一整串文本
- 中文期刊若 DOI 不返回可解析 BibTeX，可保留中文作者姓名，但仍要使用标准 `and` 连接

### 6. 处理无 DOI 条目

对无 DOI 条目，按以下顺序处理：

1. 先按完整题名搜索 Crossref
2. 对计算机视觉/机器学习论文，再查 arXiv DOI `10.48550/arXiv.*`
3. 如果题名搜索到正式会议 DOI，用正式会议记录替换原临时写法
4. 如果仍无正式 DOI，再根据来源保留为：
   - `@misc`：预印本
   - `@manual`：软件/文档
   - `@book` / `@incollection`：确实属于书籍或章节时

### 7. 特殊条目处理

**arXiv 预印本：**

- 可使用 `10.48550/arXiv.*`
- 一般整理为 `@misc`
- `publisher = {arXiv}`
- `url = {https://arxiv.org/abs/<id>}`

**软件或手册：**

- 优先使用 `@manual`
- 用 `organization` 或 `publisher` 表示机构
- 不要伪装成期刊论文

**中文期刊 DOI：**

- 如果 DOI 不返回标准 BibTeX，而是跳到中转页，手工提取题名、作者、期刊、卷期页码
- 若能确认是正式期刊，优先整理为 `@article`

### 8. 收尾校验

修改完成后，至少检查：

- 所有 cite key 是否保持稳定
- 非手册条目是否都具备 `doi`
- 是否还残留大量 `@misc + note` 的伪期刊条目
- 是否还存在 `&amp;`、乱码、HTML 实体、奇怪的长破折号
- 是否还有 `collection`、`series` 等明显无助于当前稿件输出的字段
- 会议论文和书章节的条目类型是否合理
- 作者字段是否为 BibTeX 可解析的标准格式

## 推荐输出方式

如果用户要你直接改文件：

1. 先批量重建
2. 再做一轮人工清洗
3. 最后给出简短总结：修了哪些类型的问题、哪些条目是人工兜底

如果用户要你先审查不改：

- 先列出高风险问题条目
- 再说明哪些可以通过 DOI 自动重建
- 最后指出少数需要人工确认的条目

## 完成标准

- `.bib` 的主要条目已按 DOI 或官方记录重建
- 作者姓名、期刊/会议字段、卷期页码、DOI、URL 的位置正确
- 大部分不规范的 `@misc + note` 条目已转成更合适的标准类型
- 只有确实无法标准化的条目才保留为手工整理形式
