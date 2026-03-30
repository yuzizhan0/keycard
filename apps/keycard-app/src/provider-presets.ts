/** Canonical `provider` strings + localized labels for the preset dropdown. */

export type PresetRow = {
  value: string;
  en: string;
  zh: string;
  ja: string;
  ko: string;
};

export const PROVIDER_PRESET_ROWS: readonly PresetRow[] = [
  {
    value: "OpenAI",
    en: "OpenAI (GPT-4, o-series, …)",
    zh: "OpenAI（GPT-4、o 系列等）",
    ja: "OpenAI（GPT-4、oシリーズ など）",
    ko: "OpenAI(GPT-4, o 시리즈 등)",
  },
  {
    value: "Anthropic",
    en: "Anthropic (Claude)",
    zh: "Anthropic（Claude）",
    ja: "Anthropic（Claude）",
    ko: "Anthropic(Claude)",
  },
  {
    value: "Google Gemini",
    en: "Google Gemini",
    zh: "Google Gemini",
    ja: "Google Gemini",
    ko: "Google Gemini",
  },
  {
    value: "Microsoft Azure OpenAI",
    en: "Microsoft Azure OpenAI",
    zh: "微软 Azure OpenAI",
    ja: "Microsoft Azure OpenAI",
    ko: "Microsoft Azure OpenAI",
  },
  {
    value: "Meta Llama",
    en: "Meta (Llama)",
    zh: "Meta（Llama）",
    ja: "Meta（Llama）",
    ko: "Meta(Llama)",
  },
  {
    value: "Mistral AI",
    en: "Mistral AI",
    zh: "Mistral AI",
    ja: "Mistral AI",
    ko: "Mistral AI",
  },
  {
    value: "Cohere",
    en: "Cohere",
    zh: "Cohere",
    ja: "Cohere",
    ko: "Cohere",
  },
  {
    value: "xAI",
    en: "xAI (Grok)",
    zh: "xAI（Grok）",
    ja: "xAI（Grok）",
    ko: "xAI(Grok)",
  },
  {
    value: "DeepSeek",
    en: "DeepSeek",
    zh: "DeepSeek",
    ja: "DeepSeek",
    ko: "DeepSeek",
  },
  {
    value: "Tencent Hunyuan",
    en: "Tencent Hunyuan",
    zh: "腾讯混元",
    ja: "Tencent 混元",
    ko: "텐센트 혼원",
  },
  {
    value: "Alibaba Qwen",
    en: "Alibaba Qwen",
    zh: "阿里通义千问",
    ja: "Alibaba 通義千問",
    ko: "알리바바 첸원",
  },
  {
    value: "Baidu ERNIE",
    en: "Baidu ERNIE",
    zh: "百度文心",
    ja: "百度 ERNIE",
    ko: "바이두 ERNIE",
  },
  {
    value: "Zhipu GLM",
    en: "Zhipu GLM",
    zh: "智谱 GLM",
    ja: "智谱 GLM",
    ko: "즈푸 GLM",
  },
  {
    value: "Moonshot Kimi",
    en: "Moonshot (Kimi)",
    zh: "月之暗面 Kimi",
    ja: "Moonshot（Kimi）",
    ko: "Moonshot(Kimi)",
  },
  {
    value: "ByteDance Doubao",
    en: "ByteDance Doubao",
    zh: "字节豆包",
    ja: "ByteDance 豆包",
    ko: "바이트댄스 도우바오",
  },
  {
    value: "MiniMax",
    en: "MiniMax",
    zh: "MiniMax",
    ja: "MiniMax",
    ko: "MiniMax",
  },
  {
    value: "Perplexity",
    en: "Perplexity",
    zh: "Perplexity",
    ja: "Perplexity",
    ko: "Perplexity",
  },
  {
    value: "Together AI",
    en: "Together AI",
    zh: "Together AI",
    ja: "Together AI",
    ko: "Together AI",
  },
  {
    value: "Groq",
    en: "Groq",
    zh: "Groq",
    ja: "Groq",
    ko: "Groq",
  },
  {
    value: "AWS Bedrock",
    en: "AWS Bedrock",
    zh: "AWS Bedrock",
    ja: "AWS Bedrock",
    ko: "AWS Bedrock",
  },
];

export const PROVIDER_PRESET_VALUE_SET: ReadonlySet<string> = new Set(
  PROVIDER_PRESET_ROWS.map((r) => r.value),
);
