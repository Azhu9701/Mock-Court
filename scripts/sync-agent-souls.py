#!/usr/bin/env python3
"""批量将 agent/*.md 的魂数据（body 作为原版召唤词）更新到 API"""

import os, re, json, urllib.request, urllib.error, urllib.parse, sys

API = "http://127.0.0.1:3096/api/v1"
AGENT_DIR = "/Users/huyi/Desktop/agent"
SKIP_FILES = {"soul-banner-wiki.md"}

def parse_yaml_list(yaml_text, key):
    pattern = rf'^{key}:\s*\n((?:-.*\n?)*)'
    m = re.search(pattern, yaml_text, re.MULTILINE)
    if not m:
        return []
    items = []
    for line in m.group(1).split('\n'):
        item_m = re.match(r'-\s+(.*)', line)
        if item_m:
            items.append(item_m.group(1).strip().strip("'\" "))
    return items

def parse_agent_file(path):
    with open(path, encoding="utf-8") as f:
        text = f.read()
    m = re.match(r'^---\n(.*?)\n---\n', text, re.DOTALL)
    if not m:
        print(f"  SKIP: no frontmatter in {path}")
        return None
    yaml_text = m.group(1)
    body = text[m.end():].strip()

    data = {}
    for line in yaml_text.split('\n'):
        m2 = re.match(r'^(\w[\w_]*):\s*(.*)', line)
        if m2:
            key = m2.group(1)
            val = m2.group(2).strip().strip("'\"")
            if val != '':
                data[key] = val

    for list_key in ['domain', 'tags']:
        items = parse_yaml_list(yaml_text, list_key)
        if items:
            data[list_key] = items

    # 从 body 中移除 mind:/voice: 这些 YAML 多行字段，剩余部分作为 summon_prompt
    summon = body
    for field in ['mind', 'voice', 'practice_observations', '审查记录']:
        # 匹配 field: "..." 多行字符串
        pattern = rf'\n*{field}:\s*".*?(?=\n\w[\w_]*:\s|\Z)'
        summon = re.sub(pattern, '', summon, flags=re.DOTALL)
    # 清理多余空行
    summon = re.sub(r'\n{3,}', '\n\n', summon).strip()

    data['_summon_prompt'] = summon
    return data

def api_put(path, data):
    try:
        body = json.dumps(data).encode()
        req = urllib.request.Request(f"{API}{path}", data=body, method='PUT',
            headers={'Content-Type': 'application/json'})
        with urllib.request.urlopen(req) as r:
            return r.status
    except urllib.error.HTTPError as e:
        print(f"    API PUT error {e.code}: {e.read().decode()[:200]}")
        return None
    except Exception as e:
        print(f"    API PUT error: {e}")
        return None

OUTPUT_RULE = "\n\n## 输出规范（严格遵守）\n\n- 你是思想者，不是演员。你的输出是分析文本，不是剧本\n- 严禁第三人称叙事/动作/场景/神态描写\n- 直接输出观点和论证。你的风格体现在论证方式上，不体现在戏剧表演上\n"

def main():
    files = sorted(f for f in os.listdir(AGENT_DIR) if f.endswith('.md') and f not in SKIP_FILES)
    print(f"共 {len(files)} 个魂文件\n")

    ok_count = 0
    fail_count = 0

    for fname in files:
        path = os.path.join(AGENT_DIR, fname)
        name = fname.replace('.md', '')
        print(f"处理: {name}")

        data = parse_agent_file(path)
        if not data:
            fail_count += 1
            continue

        ismism = data.get('ismism_code', '')
        domain_list = data.get('domain', [])
        model = data.get('model', 'sonnet')
        desc = data.get('description', '')
        field = desc.split('|')[0].strip() if desc else ''

        # 原版召唤词 + 输出规范
        summon_prompt = data.get('_summon_prompt', '') + OUTPUT_RULE

        update_data = {
            'ismism_code': ismism,
            'field': field,
            'domains': domain_list,
            'tags': [],
            'summon_prompt': summon_prompt,
            'model': model,
        }

        status = api_put(f'/souls/{urllib.parse.quote(name)}', update_data)
        if status == 200:
            print(f"  OK: ismism={ismism}, domains={len(domain_list)}, prompt={len(summon_prompt)}chars")
            ok_count += 1
        else:
            print(f"  FAILED: status={status}")
            fail_count += 1

    print(f"\n--- 完成: OK={ok_count}, FAIL={fail_count} ---")

if __name__ == '__main__':
    main()
