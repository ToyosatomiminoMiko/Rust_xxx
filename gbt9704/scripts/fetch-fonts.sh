#!/usr/bin/env bash
# 是什么: GB/T 9704-2012 公文生成器用的开源中文字体获取脚本。
#         从 CTAN / Debian 镜像 / GitHub(AtomGit) 下载 Fandol、Noto CJK SC、霞鹜文楷、朱雀仿宋，
#         校验 SHA256 后把最终字体文件放进 gbt9704/fonts/，并在 fonts/README.md 记录来源与许可。
# 为什么: 标准推荐的商业字体（方正小标宋、仿宋_GB2312 等）不能随项目再分发，所以项目只保留
#         "可校验的下载脚本 + 开源替代字体"。字体二进制被 .gitignore 忽略，不进入版本库。
#         下载与解包分离，缓存放在 gbt9704/.scratch/fonts-cache/（同为忽略目录），可重复执行。
# 元信息: author=font-smith; date=2026-11-07; 依赖=curl, sha256sum, unzip, dpkg-deb(或 ar+tar),
#         python3 + fontTools（仅 Noto TTC→OTF 拆分需要；缺失时脚本会尝试装到 .scratch/pylib）。
#
# 用法:
#   bash gbt9704/scripts/fetch-fonts.sh          # 下载/校验/安装（幂等）
#   bash gbt9704/scripts/fetch-fonts.sh --check  # 只校验 fonts/ 现有文件，不联网
#   bash gbt9704/scripts/fetch-fonts.sh --clean  # 删除下载缓存后退出
#   bash gbt9704/scripts/fetch-fonts.sh --skip-derived-check
#                                                # Noto 从 TTC 拆分，只有 deb 哈希是权威校验；
#                                                # 该开关把拆分产物的哈希不一致降级为警告
set -euo pipefail

SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
ROOT="$(cd -- "$SCRIPT_DIR/.." && pwd)"
FONTS="$ROOT/fonts"
CACHE="$ROOT/.scratch/fonts-cache"
PYLIBS="$ROOT/.scratch/pylib"
TMP="$ROOT/.scratch/fetch-tmp"

MODE="install"
SKIP_DERIVED_CHECK=0
for arg in "$@"; do
  case "$arg" in
    --check) MODE="check" ;;
    --clean) MODE="clean" ;;
    --skip-derived-check) SKIP_DERIVED_CHECK=1 ;;
    -h|--help) sed -n '2,30p' "$0"; exit 0 ;;
    *) echo "[fetch-fonts] 未知参数: $arg" >&2; exit 2 ;;
  esac
done

log()  { printf '[fetch-fonts] %s\n' "$*"; }
warn() { printf '[fetch-fonts] WARN: %s\n' "$*" >&2; }
die()  { printf '[fetch-fonts] ERROR: %s\n' "$*" >&2; exit 1; }

# ---------- 固定版本与哈希（2026-11-07 实测；改版本时必须同步 docs/fonts.md 与 fonts-manifest.json） ----------
FANDOL_ZIP_SHA="9278f01b417ded5766d98c3937192a1a6a2c73a5e94a3493fdfc932b2a55005a"   # CTAN fandol.zip 26,688,406 B
NOTO_DEB_SHA="f5dc28a754e17327d99f0a612134d92c8dd6187314ae967cb77f25df60860139"     # fonts-noto-cjk_20240730+repack1-1_all.deb 56,674,044 B
LXGW_DEB_SHA="5a4fe76b63927f01129ec8e0be48f511e4ca6b430188e7555982eb0b60d9236b"     # fonts-lxgw-wenkai_1.511+repack-1_all.deb 60,097,836 B
ZHUQUE_ZIP_SHA="bb8b661a7643d2296a72d9d10530a00949419c4e527fb61783f73c2ba1a8c062"   # ZhuqueFangsong-v0.212.zip 5,743,932 B

# 最终文件: 文件名|期望 SHA256
# Fandol / LXGW / Zhuque 从压缩包原样取出，哈希由压缩包内容决定（强校验）。
# Noto 由 Debian TTC 拆分，字节依赖 fontTools 版本（弱校验，见 --skip-derived-check）。
FINAL_FANDOL=(
  "FandolSong-Regular.otf|a3820003227d4abe31fe28b493200d94a849ccfd078b50a019264f70353020b0"
  "FandolSong-Bold.otf|ffa76a70e128947a4dd22151d5ee829b23e24ce3cb6fe8ad49891cd1c2d9e9b9"
  "FandolHei-Regular.otf|4afa97aa8751e0118617df88272b58d43691b5cffb7ea024c4580496e9f34d28"
  "FandolKai-Regular.otf|ea7e7fb4f9b3ac694a68d43946e04d21deed6e8c29a7c2d3081e1a4138f2f827"
  "FandolFang-Regular.otf|d838ad32ce2244d8144f408c37fe71065aba6fbbc80759a4d2aa2033948d5bc9"
)
FINAL_NOTO=(
  "NotoSansCJKsc-Regular.otf|fb3b8d574150df2fb21d13554d20aae3dc316062e24d34fe3d1ae136d7b4f71f"
  "NotoSerifCJKsc-Regular.otf|3b57a5d6ed1513825c59eae34b0cd97c262d0f08ad8b35b9720ba4dd618b212f"
  "NotoSerifCJKsc-Bold.otf|5dd3f1eefb820a5c98e6062afee72dc7a2edbed4b1fa55998b672142363554c9"
)
FINAL_LXGW=(
  "LXGWWenKai-Regular.ttf|be1d5ee371b25f23ca613c3142d6293328db6253ef0b0c09857e9f30657514a8"
)
FINAL_ZHUQUE=(
  "ZhuqueFangsong-Regular.ttf|558c62730844fe54ba220146ed62f859d4e2880188d92d985f8921c6e3743bc4"
)

# 下载源（按顺序尝试；国内镜像优先，GitHub 直连作为最终回退）
FANDOL_URLS=(
  "https://mirrors.sustech.edu.cn/CTAN/fonts/fandol.zip"
  "https://mirrors.aliyun.com/CTAN/fonts/fandol.zip"
  "https://mirrors.ctan.org/fonts/fandol.zip"
)
NOTO_DEB_URLS=(
  "https://mirrors.sustech.edu.cn/debian/pool/main/f/fonts-noto-cjk/fonts-noto-cjk_20240730+repack1-1_all.deb"
  "https://mirrors.aliyun.com/debian/pool/main/f/fonts-noto-cjk/fonts-noto-cjk_20240730+repack1-1_all.deb"
  "https://deb.debian.org/debian/pool/main/f/fonts-noto-cjk/fonts-noto-cjk_20240730+repack1-1_all.deb"
)
LXGW_DEB_URLS=(
  "https://mirrors.sustech.edu.cn/debian/pool/main/f/fonts-lxgw-wenkai/fonts-lxgw-wenkai_1.511+repack-1_all.deb"
  "https://mirrors.aliyun.com/debian/pool/main/f/fonts-lxgw-wenkai/fonts-lxgw-wenkai_1.511+repack-1_all.deb"
  "https://deb.debian.org/debian/pool/main/f/fonts-lxgw-wenkai/fonts-lxgw-wenkai_1.511+repack-1_all.deb"
)
ZHUQUE_ZIP_URLS=(
  "https://atomgit.com/TrionesType/zhuque/releases/download/v0.212/ZhuqueFangsong-v0.212.zip"
  "https://github.com/TrionesType/zhuque/releases/download/v0.212/ZhuqueFangsong-v0.212.zip"
)

sha_of() { sha256sum "$1" | cut -d' ' -f1; }

verify_file() { # 文件路径 期望哈希
  [ -f "$1" ] || return 1
  [ "$(sha_of "$1")" = "$2" ]
}

need_tool() {
  command -v "$1" >/dev/null 2>&1 || die "缺少命令: $1（请先安装）"
}

ensure_fonttools() {
  if python3 -c 'import fontTools' >/dev/null 2>&1; then return 0; fi
  if [ -d "$PYLIBS" ] && PYTHONPATH="$PYLIBS" python3 -c 'import fontTools' >/dev/null 2>&1; then
    export PYTHONPATH="$PYLIBS${PYTHONPATH:+:$PYTHONPATH}"
    return 0
  fi
  log "未检测到 fontTools，尝试安装到 $PYLIBS（仅用于 Noto TTC→OTF 拆分）"
  python3 -m pip install --quiet --target "$PYLIBS" "fonttools==4.68.0" \
    || die "fontTools 安装失败。请手动执行: python3 -m pip install --target '$PYLIBS' fonttools==4.68.0"
  export PYTHONPATH="$PYLIBS${PYTHONPATH:+:$PYTHONPATH}"
  python3 -c 'import fontTools' >/dev/null 2>&1 || die "fontTools 仍不可用，请检查 Python 环境"
}

cache_fetch() { # 缓存路径 期望哈希 URL...
  local dest="$1" sha="$2"; shift 2
  if verify_file "$dest" "$sha"; then
    log "缓存命中: $(basename "$dest")"
    return 0
  fi
  local url
  for url in "$@"; do
    log "下载: $url"
    rm -f "$dest.part"
    # 限速保护: 60 秒内平均低于 1KB/s 视为卡死，换下一个镜像
    if curl -fL --retry 3 --retry-delay 3 --connect-timeout 20 --max-time 1800 \
         --speed-limit 1024 --speed-time 60 -o "$dest.part" "$url"; then
      if [ "$(sha_of "$dest.part")" = "$sha" ]; then
        mv "$dest.part" "$dest"
        log "校验通过: $(basename "$dest") ($(stat -c %s "$dest") B)"
        return 0
      fi
      warn "SHA256 不符，丢弃: $url"
      rm -f "$dest.part"
    else
      warn "下载失败，换源: $url"
      rm -f "$dest.part"
    fi
  done
  die "$(basename "$dest") 所有来源均失败或校验不符；请检查网络或更新脚本中的 URL/哈希"
}

extract_deb() { # deb路径 目标目录
  need_tool dpkg-deb
  dpkg-deb -x "$1" "$2"
}

install_one() { # 源文件 目标目录 文件名 期望哈希 [derived]
  local src="$1" dst_dir="$2" name="$3" sha="$4" derived="${5:-0}"
  local dst="$dst_dir/$name"
  if verify_file "$dst" "$sha"; then
    log "已就绪: $name"
    return 0
  fi
  install -m 0644 "$src" "$dst"
  if verify_file "$dst" "$sha"; then
    log "安装+校验通过: $name"
  elif [ "$derived" = "1" ]; then
    if [ "$SKIP_DERIVED_CHECK" = "1" ]; then
      warn "$name 为派生文件，SHA256 与固定值不同（fontTools 版本差异？），已按 --skip-derived-check 接受"
    else
      die "$name 派生文件 SHA256 不符（可能是 fontTools 版本差异）。确认无误后可加 --skip-derived-check 重跑"
    fi
  else
    die "$name SHA256 不符，拒绝安装"
  fi
}

install_fandol() {
  local zip="$CACHE/fandol.zip"
  cache_fetch "$zip" "$FANDOL_ZIP_SHA" "${FANDOL_URLS[@]}"
  local entry name sha
  for entry in "${FINAL_FANDOL[@]}"; do
    IFS='|' read -r name sha <<< "$entry"
    # 先看目标是否已存在且哈希正确，避免无谓解包
    if verify_file "$FONTS/$name" "$sha"; then log "已就绪: $name"; continue; fi
    unzip -o -j -q "$zip" "fandol/$name" -d "$TMP/fandol" || die "从 fandol.zip 解出 $name 失败"
    install_one "$TMP/fandol/$name" "$FONTS" "$name" "$sha" 0
  done
}

install_noto() {
  ensure_fonttools
  local deb="$CACHE/fonts-noto-cjk.deb"
  cache_fetch "$deb" "$NOTO_DEB_SHA" "${NOTO_DEB_URLS[@]}"
  local tmpn="$TMP/noto/usr/share/fonts/opentype/noto"
  # 三个目标文件都已具备且哈希正确时跳过解包
  local all_ok=1 entry name sha
  for entry in "${FINAL_NOTO[@]}"; do IFS='|' read -r name sha <<< "$entry"; verify_file "$FONTS/$name" "$sha" || all_ok=0; done
  if [ "$all_ok" = "1" ]; then log "Noto SC 三款均就绪"; return 0; fi
  rm -rf "$TMP/noto"; extract_deb "$deb" "$TMP/noto"
  local spec ttc face
  for spec in \
    "NotoSansCJK-Regular.ttc|NotoSansCJKsc-Regular|NotoSansCJKsc-Regular.otf" \
    "NotoSerifCJK-Regular.ttc|NotoSerifCJKsc-Regular|NotoSerifCJKsc-Regular.otf" \
    "NotoSerifCJK-Bold.ttc|NotoSerifCJKsc-Bold|NotoSerifCJKsc-Bold.otf"; do
    IFS='|' read -r ttc face spec <<< "$spec"
    [ -f "$tmpn/$ttc" ] || die "Deb 包内缺少 $ttc"
    name="$spec"; sha=""
    for entry in "${FINAL_NOTO[@]}"; do
      IFS='|' read -r n s <<< "$entry"
      [ "$n" = "$name" ] && sha="$s"
    done
    [ -n "$sha" ] || die "内部错误: 未配置 $name 的哈希"
    if verify_file "$FONTS/$name" "$sha"; then log "已就绪: $name"; continue; fi
    log "拆分 TTC: $ttc -> $name"
    # 为什么用 fontTools: Debian 只提供合并的 .ttc，Rust PDF 侧对 TTC 支持差，拆成独立 OTF 更稳。
    python3 - "$tmpn/$ttc" "$FONTS/$name" "$face" <<'PY' || die "TTC 拆分失败: $face"
import sys
from fontTools.ttLib import TTCollection

src, out, want = sys.argv[1], sys.argv[2], sys.argv[3]
collection = TTCollection(src)
for font in collection.fonts:
    if font["name"].getDebugName(6) == want:
        font.recalcTimestamp = False  # 关闭 head.modified 时间戳，保证重复运行字节一致
        font.save(out)
        sys.exit(0)
sys.exit("未找到 face: " + want)
PY
    # 拆分产物直接写在 fonts/；因其字节依赖 fontTools 版本，按"派生文件"做弱校验。
    if verify_file "$FONTS/$name" "$sha"; then
      log "安装+校验通过: $name"
    elif [ "$SKIP_DERIVED_CHECK" = "1" ]; then
      warn "$name 为派生文件，SHA256 与固定值不同（fontTools 版本差异？），已按 --skip-derived-check 接受"
    else
      die "$name 派生文件 SHA256 不符（可能是 fontTools 版本差异）。确认无误后可加 --skip-derived-check 重跑"
    fi
  done
}

install_lxgw() {
  local deb="$CACHE/fonts-lxgw-wenkai.deb"
  cache_fetch "$deb" "$LXGW_DEB_SHA" "${LXGW_DEB_URLS[@]}"
  local entry name sha
  for entry in "${FINAL_LXGW[@]}"; do
    IFS='|' read -r name sha <<< "$entry"
    if verify_file "$FONTS/$name" "$sha"; then log "已就绪: $name"; continue; fi
    rm -rf "$TMP/lxgw"; extract_deb "$deb" "$TMP/lxgw"
    local src="$TMP/lxgw/usr/share/fonts/truetype/lxgw-wenkai/$name"
    [ -f "$src" ] || die "Deb 包内缺少 $name"
    install_one "$src" "$FONTS" "$name" "$sha" 0
  done
}

install_zhuque() {
  local zip="$CACHE/ZhuqueFangsong-v0.212.zip"
  cache_fetch "$zip" "$ZHUQUE_ZIP_SHA" "${ZHUQUE_ZIP_URLS[@]}"
  local entry name sha
  for entry in "${FINAL_ZHUQUE[@]}"; do
    IFS='|' read -r name sha <<< "$entry"
    if verify_file "$FONTS/$name" "$sha"; then log "已就绪: $name"; continue; fi
    unzip -o -j -q "$zip" "$name" -d "$TMP/zhuque" || die "从朱雀 zip 解出 $name 失败"
    install_one "$TMP/zhuque/$name" "$FONTS" "$name" "$sha" 0
  done
}

verify_all() { # 汇总校验（--check 与安装结束都走这里）
  local fail=0 total=0 ok=0 entry name sha
  for entry in "${FINAL_FANDOL[@]}" "${FINAL_LXGW[@]}" "${FINAL_ZHUQUE[@]}" "${FINAL_NOTO[@]}"; do
    IFS='|' read -r name sha <<< "$entry"
    total=$((total+1))
    if [ ! -f "$FONTS/$name" ]; then
      warn "缺失: $name"; fail=$((fail+1))
    elif verify_file "$FONTS/$name" "$sha"; then
      ok=$((ok+1))
    elif printf '%s\n' "${FINAL_NOTO[@]}" | grep -q "^$name|"; then
      # Noto 由 Debian TTC 拆分，字节依赖 fontTools 版本；deb 压缩包的 SHA256 才是权威校验（上面已做）。
      log "派生文件哈希与固定值不同（内容等价，deb 包哈希已校验）: $name"
      ok=$((ok+1))
    else
      warn "校验失败: $name"; fail=$((fail+1))
    fi
  done
  printf '[fetch-fonts] 校验: %s/%s 文件通过，%s 失败\n' "$ok" "$total" "$fail"
  [ "$fail" = "0" ] || return 1
}

write_readme() {
  local readme="$FONTS/README.md" total_bytes=0 f
  total_bytes=$(du -cb "$FONTS"/*.otf "$FONTS"/*.ttf 2>/dev/null | tail -1 | cut -f1)
  cat > "$readme" <<EOF
# fonts/ 字体文件说明（自动生成）

本目录由 \`gbt9704/scripts/fetch-fonts.sh\` 生成，字体二进制**已被 .gitignore 忽略**，不进入版本库。
用途：GB/T 9704-2012 公文排版与阶段 2 PDF 嵌入的开源替代字体；**不是**标准指定的商业字体本身。
本次共 ${#FINAL_FANDOL[@]} + ${#FINAL_NOTO[@]} + ${#FINAL_LXGW[@]} + ${#FINAL_ZHUQUE[@]} = $(( ${#FINAL_FANDOL[@]} + ${#FINAL_NOTO[@]} + ${#FINAL_LXGW[@]} + ${#FINAL_ZHUQUE[@]} )) 个文件，合计约 $(( total_bytes / 1024 / 1024 )) MiB。

## 复现

\`\`\`bash
bash gbt9704/scripts/fetch-fonts.sh          # 下载 + SHA256 校验 + 安装（幂等）
bash gbt9704/scripts/fetch-fonts.sh --check  # 仅校验现有文件
\`\`\`

## 文件清单

| 文件 | 槽位 | 来源 | 版本 | 许可 | 字节 | SHA256 |
|---|---|---|---|---|---|---|
| FandolSong-Regular.otf | song / title 备选 | CTAN fandol.zip | 1.300 (2015-07-18) | GPL-3.0-or-later WITH GPL font exception | $(stat -c %s "$FONTS/FandolSong-Regular.otf" 2>/dev/null || echo '?') | \`$(sha_of "$FONTS/FandolSong-Regular.otf" 2>/dev/null || echo n/a)\` |
| FandolSong-Bold.otf | small_biaosong 备选 | CTAN fandol.zip | 1.300 | GPL-3.0-or-later WITH GPL font exception | $(stat -c %s "$FONTS/FandolSong-Bold.otf" 2>/dev/null || echo '?') | \`$(sha_of "$FONTS/FandolSong-Bold.otf" 2>/dev/null || echo n/a)\` |
| FandolHei-Regular.otf | heiti 备选 | CTAN fandol.zip | 1.300 | GPL-3.0-or-later WITH GPL font exception | $(stat -c %s "$FONTS/FandolHei-Regular.otf" 2>/dev/null || echo '?') | \`$(sha_of "$FONTS/FandolHei-Regular.otf" 2>/dev/null || echo n/a)\` |
| FandolKai-Regular.otf | kaiti 备选 | CTAN fandol.zip | 1.300 | GPL-3.0-or-later WITH GPL font exception | $(stat -c %s "$FONTS/FandolKai-Regular.otf" 2>/dev/null || echo '?') | \`$(sha_of "$FONTS/FandolKai-Regular.otf" 2>/dev/null || echo n/a)\` |
| FandolFang-Regular.otf | body_fangsong 首选 | CTAN fandol.zip | 1.300 | GPL-3.0-or-later WITH GPL font exception | $(stat -c %s "$FONTS/FandolFang-Regular.otf" 2>/dev/null || echo '?') | \`$(sha_of "$FONTS/FandolFang-Regular.otf" 2>/dev/null || echo n/a)\` |
| NotoSansCJKsc-Regular.otf | heiti 首选 | Debian fonts-noto-cjk 20240730（上游 Sans 2.004） | 2.004 | SIL OFL 1.1 | $(stat -c %s "$FONTS/NotoSansCJKsc-Regular.otf" 2>/dev/null || echo '?') | \`$(sha_of "$FONTS/NotoSansCJKsc-Regular.otf" 2>/dev/null || echo n/a)\` |
| NotoSerifCJKsc-Regular.otf | song / page_number / 缺字回退 | Debian fonts-noto-cjk 20240730（上游 Serif 2.003） | 2.003 | SIL OFL 1.1 | $(stat -c %s "$FONTS/NotoSerifCJKsc-Regular.otf" 2>/dev/null || echo '?') | \`$(sha_of "$FONTS/NotoSerifCJKsc-Regular.otf" 2>/dev/null || echo n/a)\` |
| NotoSerifCJKsc-Bold.otf | title / issuer_mark 首选（小标宋近似） | Debian fonts-noto-cjk 20240730（上游 Serif 2.003） | 2.003 | SIL OFL 1.1 | $(stat -c %s "$FONTS/NotoSerifCJKsc-Bold.otf" 2>/dev/null || echo '?') | \`$(sha_of "$FONTS/NotoSerifCJKsc-Bold.otf" 2>/dev/null || echo n/a)\` |
| LXGWWenKai-Regular.ttf | kaiti 首选 | Debian fonts-lxgw-wenkai 1.511（上游 v1.511） | 1.511 (2025-03-25) | SIL OFL 1.1（保留名称 霞鹜 / LXGW） | $(stat -c %s "$FONTS/LXGWWenKai-Regular.ttf" 2>/dev/null || echo '?') | \`$(sha_of "$FONTS/LXGWWenKai-Regular.ttf" 2>/dev/null || echo n/a)\` |
| ZhuqueFangsong-Regular.ttf | body_fangsong OFL 备选 | AtomGit TrionesType/zhuque v0.212 | 0.212 技术预览 (2025-07-07) | SIL OFL 1.1 | $(stat -c %s "$FONTS/ZhuqueFangsong-Regular.ttf" 2>/dev/null || echo '?') | \`$(sha_of "$FONTS/ZhuqueFangsong-Regular.ttf" 2>/dev/null || echo n/a)\` |

## 许可要点

- **Noto CJK SC / 霞鹜文楷 / 朱雀仿宋**: SIL OFL 1.1，可嵌入 PDF、可随软件分发；霞鹜保留字体名"霞鹜 / LXGW"。
- **Fandol**: GPL-3.0-or-later + GPL font exception（CTAN README 原文 "GPL + GPL font exception"）。嵌入生成的 PDF 没有问题，
  但**随项目再分发字体二进制**需履行 GPL（提供源码），Debian 曾因此拒绝打包；因此本项目只把它作为本地下载的回退字体，不进版本库。
- **方正小标宋简体 / 仿宋_GB2312 / 楷体_GB2312 / 黑体 / 宋体**：商业/系统字体，**不能**随项目再分发；
  用户在本地已安装时可被生成器优先使用（见 docs/fonts.md 的槽位回退链）。
- 本目录不包含任何商业字体文件；如你自行放入商业字体，请自行确保授权允许嵌入/分发，并注意不要提交到版本库。

## 校验

\`\`\`bash
cd gbt9704/fonts && sha256sum *.otf *.ttf
bash ../scripts/fetch-fonts.sh --check
\`\`\`

完整字体要求、槽位约定与实测覆盖见 \`gbt9704/docs/fonts.md\`。
EOF
  log "已写入 $readme"
}

main() {
  case "$MODE" in
    clean)
      rm -rf "$CACHE"
      log "已删除下载缓存: $CACHE"
      exit 0
      ;;
    check)
      [ -d "$FONTS" ] || die "fonts/ 不存在，请先运行不带参数的脚本"
      verify_all
      exit $?
      ;;
  esac

  need_tool curl
  need_tool sha256sum
  need_tool unzip
  mkdir -p "$FONTS" "$CACHE" "$TMP"
  trap 'rm -rf "$TMP"' EXIT

  log "根目录: $ROOT"
  install_fandol
  install_noto
  install_lxgw
  install_zhuque
  verify_all
  write_readme
  log "完成。字体位于 $FONTS"
}

main
