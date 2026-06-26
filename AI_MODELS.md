# AI model testleri ve öneriler

keencli `analyze` çıktısı için OpenRouter modellerinin karşılaştırmalı test sonuçları.

**Test verisi:** `outputs/2026-06-26_14-45-35/` (Hopper_Fiber, filtrelenmiş log ~62 satır)  
**Prompt:** `prompt_for_ai.txt` (tüm modeller aynı girdi)  
**Sıcaklık:** `LLM_TEMPERATURE` tanımlı değilse **0.3**

Model seçimi ve `.env` örnekleri: [README — AI analizi](README.md#ai-analizi-opsiyonel)

---

## Referans bulgular (loglardan)

| Severity | Beklenen bulgu |
|----------|----------------|
| **HIGH** | 24 Haziran 00:20–00:36: **4** ping-check flap + her seferinde DNS silme/ekleme |
| **MEDIUM** | 23 Haziran ~08:59: WAN fiziksel düşüş, Modem hangup, **IP + default route kaybı**, toparlanma |
| **LOW** | GigabitEthernet0/1 flap (24/25/26, 100FD), ndnproxy, WSD |

---

## Genel sıralama

| Sıra | Model | Puan | Özet |
|------|-------|------|------|
| 1 | `anthropic/claude-sonnet-4.6` | ~9/10 | En eksiksiz; DNS korelasyonu, doğru severity |
| 2 | `deepseek/deepseek-v4-pro` | ~7.8/10 | Öz ve güvenilir; ucuz segmentte en iyi |
| 3 | `google/gemini-2.5-pro` | ~7.5/10 | İyi detay; severity biraz şişkin |
| 4 | `qwen/qwen3.5-plus-02-15` | ~7–7.5/10 | DNS iyi; Jun 23 WAN bazen atlanır |
| 5 | `qwen/qwen3.7-max` | ~7.2/10 | Ana hikâye doğru; DNS ve 4. flap eksik |
| 6 | `openai/gpt-4.1` | ~5.5/10 | HIGH kaçırıldı; route kaybı inkâr |
| 7 | `x-ai/grok-4.20` | ~5/10 | Ping-check flap yok sayıldı |
| 8 | `qwen/qwen3-235b-a22b` | ~4.5/10 | HIGH kaçırıldı; route kaybı inkâr |
| 9 | `x-ai/grok-4.3` | ~4/10 | Çoğu bulgu eksik |

**Test edilmedi / erişilemedi**

| Model | Durum |
|-------|--------|
| `sakana/fugu-ultra` | OpenRouter 403 — hesap erişimi kapalı |

---

## Kriter matrisi

Üst sıra modeller için ayrıntılı değerlendirme; «önerilmez» modeller için bkz. aşağı.

| Kriter | Claude | DeepSeek | Gemini | Qwen 3.7 | Qwen 3.5 | GPT-4.1 | Grok 4.20 | Qwen 235B | Grok 4.3 |
|--------|--------|----------|--------|----------|----------|---------|-----------|-----------|----------|
| 4 ping-check flap | ✓ | ✓ | △ | △ | ✓ | ✳ | ✗ | ✗ | ✗ |
| DNS korelasyonu | ✓ | ✓ | △ | ✗ | ✓ | △ | ✗ | ✗ | ✗ |
| Jun 23 route/IP | ✓ | △ | ✓ | ✓ | △ | ✗ | ✗ | ✗ | ✓ |
| LAN flap 26 Haz | ✓ | ✓ | ✓ | ✓ | — | ✓ | ✓ | ✓ | ✗ |
| Severity doğru | ✓ | ✓ | △ | △ | △ | ✗ | △ | ✗ | ✗ |

✳ = yanlış severity · △ = kısmen · — = ayrı fetch klasöründe test

---

## Önerilmez modeller (ayrıntı)

### `qwen/qwen3-235b-a22b`

- **HIGH: none** — 24 Haziran 4 ping-check flap + DNS tamamen atlandı
- Jun 23 hangup **MEDIUM**'da ama «route loss yok» — logda IP cleared + default route removed var
- Özet'te «0 pingcheck failures» — geçmiş log flap'lerini yok sayıyor
- LAN flap (24–26) ve ndnproxy/WSD **LOW**'da doğru
- Büyük parametre sayısına rağmen GPT-4.1 / Grok ile benzer eksiklikler

### `openai/gpt-4.1`

- **HIGH: none** — 4 flap + DNS prompt'a göre yanlış
- Ping-check **LOW**'a indirildi
- «Route loss yok» — logda IP cleared + default route removed var
- Formata ekstra system JSON eklendi

### `x-ai/grok-4.20` / `x-ai/grok-4.3`

Ortak sorun: `pingcheck.successcount` anlık sayacına takılıp **geçmiş log flap'lerini yok sayma**.

| | Grok 4.20 | Grok 4.3 |
|--|-----------|----------|
| Ping-check HIGH | ✗ | ✗ |
| LAN flap | ✓ LOW | ✗ |
| Jun 23 hangup | özet | ✓ MEDIUM |

---

## Notlar

Raporlar `outputs/TARİH/ai_report_MODEL.md` olarak kaydedilir; aynı model tekrar çalışınca **üzerine yazılır**. Karşılaştırma için önce kopyalayın:

```bash
cp outputs/.../ai_report_MODEL.md outputs/.../ai_report_MODEL-kopya.md
```

---

*Son güncelleme: 2026-06-26 — keencli v1.0.6*