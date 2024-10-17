<script lang="ts">
  import { onMount } from "svelte";

  /// Schema for API response
  interface Tag {
    name: string;
    url: string;
  }

  interface Paper {
    slug: string;
    paper_url: string;
    code_url: string;
    image_url: string;
    title: string;
    github_repo: string;
    framework: string;
    publish_date: string;
    description: string;
    tags: Tag[] | null;
  }

  let papers: Paper[] = [];
  let loading: boolean = true;
  let error: any = null;

  /// This gets triggered when the component is mounted
  onMount(async () => {
    try {
      const response = await fetch("/api/pwc");
      if (!response.ok) {
        throw new Error(`HTTP error! status: ${response.status}`);
      }
      const data = await response.json();

      if (!data || !Array.isArray(data)) {
        throw new Error("Invalid data received from API");
      }

      papers = data.filter((paper) => paper && paper.title && paper.paper_url);
    } catch (err) {
      console.error("Error fetching papers:", err);
      // error = err.message;
      error = null;
    } finally {
      loading = false;
      papers = [
        {
          slug: "/paper/f5-tts-a-fairytaler-that-fakes-fluent-and",
          paper_url:
            "https://paperswithcode.com/paper/f5-tts-a-fairytaler-that-fakes-fluent-and",
          code_url:
            "https://paperswithcode.com/paper/f5-tts-a-fairytaler-that-fakes-fluent-and#code",
          image_url:
            "data:image/jpeg;base64,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",
          title:
            "F5-TTS: A Fairytaler that Fakes Fluent and Faithful Speech with Flow Matching",
          github_repo: "https://github.com/SWivid/F5-TTS",
          framework:
            "https://production-assets.paperswithcode.com/perf/images/frameworks/pytorch-2fbf2cb9.png",
          publish_date: "9 Oct 2024",
          description:
            "This sampling strategy for flow step can be easily applied to existing flow matching based models without retraining.",
          tags: [
            {
              name: "Denoising",
              url: "https://paperswithcode.com/task/denoising",
            },
            {
              name: "Text to Speech",
              url: "https://paperswithcode.com/task/text-to-speech",
            },
          ],
        },
        {
          slug: "/paper/pyramidal-flow-matching-for-efficient-video",
          paper_url:
            "https://paperswithcode.com/paper/pyramidal-flow-matching-for-efficient-video",
          code_url:
            "https://paperswithcode.com/paper/pyramidal-flow-matching-for-efficient-video#code",
          image_url:
            "https://production-media.paperswithcode.com/thumbnails/papergithubrepo/a36ceb99-bcb0-480c-ad0a-11d870980423.jpg",
          title:
            "Pyramidal Flow Matching for Efficient Video Generative Modeling",
          github_repo: "https://github.com/jy0205/Pyramid-Flow",
          framework:
            "https://production-assets.paperswithcode.com/perf/images/frameworks/pytorch-2fbf2cb9.png",
          publish_date: "8 Oct 2024",
          description:
            "Video generation requires modeling a vast spatiotemporal space, which demands significant computational resources and data usage.",
          tags: [
            {
              name: "Text-to-Video Generation",
              url: "https://paperswithcode.com/task/text-to-video-generation",
            },
            {
              name: "Video Generation",
              url: "https://paperswithcode.com/task/video-generation",
            },
          ],
        },
        {
          slug: "/paper/diffusion-for-world-modeling-visual-details",
          paper_url:
            "https://paperswithcode.com/paper/diffusion-for-world-modeling-visual-details",
          code_url:
            "https://paperswithcode.com/paper/diffusion-for-world-modeling-visual-details#code",
          image_url:
            "data:image/jpeg;base64,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",
          title: "Diffusion for World Modeling: Visual Details Matter in Atari",
          github_repo: "https://github.com/eloialonso/diamond",
          framework:
            "https://production-assets.paperswithcode.com/perf/images/frameworks/pytorch-2fbf2cb9.png",
          publish_date: "20 May 2024",
          description:
            "Motivated by this paradigm shift, we introduce DIAMOND (DIffusion As a Model Of eNvironment Dreams), a reinforcement learning agent trained in a diffusion world model.",
          tags: [
            {
              name: "Image Generation",
              url: "https://paperswithcode.com/task/image-generation",
            },
            {
              name: "reinforcement-learning",
              url: "https://paperswithcode.com/task/reinforcement-learning-2",
            },
            {
              name: "",
              url: "https://paperswithcode.com/paper/diffusion-for-world-modeling-visual-details#tasks",
            },
          ],
        },
        {
          slug: "/paper/representation-alignment-for-generation",
          paper_url:
            "https://paperswithcode.com/paper/representation-alignment-for-generation",
          code_url:
            "https://paperswithcode.com/paper/representation-alignment-for-generation#code",
          image_url:
            "data:image/jpeg;base64,/9j/4AAQSkZJRgABAQAAAQABAAD/2wBDAAoKCgoKCgsMDAsPEA4QDxYUExMUFiIYGhgaGCIzICUgICUgMy03LCksNy1RQDg4QFFeT0pPXnFlZXGPiI+7u/v/wgALCACcAPIBAREA/8QAGgABAAMBAQEAAAAAAAAAAAAAAAECAwQFB//aAAgBAQAAAAD7MzwvSN7iZFOeUa6BYIACQgASHP5ukY6RefVkMfLFs7q9vcGPlWvvtEW6JDHx4tEWW6duwMuCnVvdC0yMfMrMXrMOr0DPQK8/RYBh460VSjv9Ew3cfNE1tEdfYGHjwiQ7fTMdnJ5ts9cbW6/RDHx9J00hMdPSYbuHzwT1+kGHlVshC3X6Jlq4OBW0J6/TDHyEpRDt9A5ulw+fIpPd6IYeTNaXgju9I5+hzZkyrtuGHjXotEJ7vRMds+N0JsrG1hh5GsRInp9Apfl8nSkaRER6PeMPHibUmYd3ohycFF6TbLXt7Rl49kxFkdfoByeXE2gPQ7hj5kDTVHR1By+Ulbalo7esZePVJEz2eiHNwTRpES7eoYeNMm0zXr7QQASCAkAAAAAA/8QARBAAAQIEBAMDCQYCCAcAAAAAAQIRAAMSIQQiMUETUWEgMnEUQlJTgZGSodIQI2JyscEzggUVMEOywtHhJVBjg6Li8P/aAAgBAQABPwD7JsxEpNS3Z9hAxshW6vamFY6QBSCpmcmk3jy3Du1RdvRMeWyPxb7QifKmJfiAD8RAMcSX61HxCOJL9aj4hHEl+tR8QjiS/Wo+IQlSVXBB8OytaZaSsuw5XMDGyDur4THlmHBFSlN0EDG4Z0grVf8ACY8tkA+d7ujxLnypossJu2a36xxJfrUfEI4kv1qPiEcSX61HxCOJL9aj4hCVoVooFuRB7bQ0MIaGENDQ0N22hoaGhoaGhobt4pKjKSUpWpQWCySBcDrBkTuIU8HEFIQKSJkCStCFAyMSSTS3EHi4e4jgTV/3E8fdqDmYNnMS5U8S01YfEA0s3FG8GXNKlPhp7gG9Y3uYRJmAk+T4q3OcCCDAkrRUgycQb+s6C14lCdJWFeS4hwNFTX1iWoqQklNJa437WIJEhZqIYbf7EQVTEqvMWvnmJPzMCYqnvzXI0BVcDdnhJWy0ibNuARmLc2N4K10AJVMaom5O1ucKWpOi16g6n3C8GYqkATJhdIUTUf8AWAtZDmYsBRYgEi7WJvFazetYZ9CQ55i+ggLXQ9czVjc7XBGbpeMGqYVLClL0diS3uJPaxC1ysPMmpF0pJDgm/gI/rOeXHDRYA9xfytH9Y4kG0lB0d0riVjcTNWEhEpibuFJYjqee0FeNSWrwju2qof8ApIKZYw4TyBLiP+IpAfyZwDuqHx/o4d3NnVD/ANIecnDMlmuq/Ptzx9xMgIUSrJdSg3QcrogBbVUlh063bJApNR4YJoZ2azajJCwqsIEogPZmb/BAdRYS7MNRy3aiAkpJFD1ADTYl7mmEpdYGjHMKf/SFoqUAlDOCdAO9sTSbQcJNopCUhyTqDrt3YwsmZJWpUwAkjof0A7U8VSljZoTIFKqjt01282BJQGdStDsD/lhpRDDCyc7+b/tAQ1+BLChcEc/GAuaRdCQwtfeHnAAGUBfZUBU0A5Eu/PURVOBfhoZvSiqfSAZaQOioS7XDHs4h+BMpd+j/ALQFYipSq5j1EtmaxgzJ6iDXNA/mEVYgZeLMcHV1N1hUydmzzRcDzm5XhM2ezErIALd57wlWJ0rVozur3nqI404pAK5hsdXcPAXPA/iTDcEOVN1fpzjiT1IU0+a4OgKmbk3XaMEpajMqUs6M7sx8ewpZC0JYZnv2lEJSSdADAxkj0lb+araEqC0gh/7HEgnDzbPaACDdDAj0X/ywAaVEAkOLFIZjcuKYTSSoLTqb5dhv3YZnZPeHogOeRywpx5jdKdnd3pgXD8N7uMov7aYZyAUX3yjU6EZYSbPQXceaNGZjkhgKshII9EC4Gtk7RgaUTJlIbKGdIHzAHYmBPFlO7uw+3GTZklMpSDS5IuAQfiIjyycxzp0GyPq32jyueQghY3csnQFnOaPLcQBaYnfZLBjzq3g4ue7CaguAQQE8nI70HGTtRMTSSbCm3JzVHlU/O01BbonX4oGMnN30gnQsn6ow09cxS0LLlIHIfIE9rFMJExzYC52+ZECkJJtrrlZjoO9vAoc0KRoXLp+qGsHYpb8IcbF6oKdVOl30NPi9lQwcl0aVG6d+ZqiuWQTUlrFnT4t3oK0VIUogP1T9UAJrOZJb8rtyOYQVpJLs2pNSfec0YFwuYOmzbnoT2FpPElnl1+3GAkSmPnOYZRSbqYEWYwmu916cjpAE0AKClM+jK8XhKZgUkVZb830eEhdR7zvfKq29oNYSKTMuepNrgRnIIzBm5xgCqpddVwOfuv2p5AkTCz87tBnKyjhKuLffubmEzJJCCAus1Onim3K7bwFyyShpgDH+931iVikSyQlBU4vVMew8YONQkkcNOo86DjkDWSjR+8x0sIOPSADwUFiXFUeXJJJEpDc6oTjk+pR8Vud4w+JE5JAQlJ5gu47E5IM2UerdT9uOAaUpQBYksQP3BikLBLAgBzYaD+WAUJqJSl0EAaAC9/NgSwQ6ikWdIZJ6N3YCEM+VmLlh4ADLBSjLQ1RA2S9v5YNBALJABdwA7kad2DQVnKDUXGn0xgQApdLMwGwvzLAX7WI/gTS7MIrCSkVMFdSzfFCiR3Cq4uXP1wlVNSnLgMG32Nq4JCqVIW6SATc2ItfNFVyxNLDe/Mh64Qq9lE2F3JAHUFcBgSKncWAVt1zwTQs0qcbXP1xUXJQTpZy4Jf8ANGBupTLfLe5+pXYXTUhwdbfbjikCU5bMWNtvEiEmSUqU6fykpfqRmMApWoqWpABFjlNvi2gKTW6KEtoTT7hmgkOFClgAxBToLP3rw8oVBJSU+KSTyHeitIAKVIcuGdL2/mjKCFGit+afq2jAlNS6WYgOA37E9rEXkTNTbaGXSbKPiFQEqEtIZT39ODWbivXep290KQqsd9ir8UfeubHR/OZjztAExKu6dCfOOsULJCVAsx9IfNoomFQSkHT8XtctBrKRlUGJvnFjtaMI4mTHfu9dSeoHYmPxpXRzq32492lBKtyDc/sRCipqRU4Ja5frvAKl00k3e9SmbbeApVBcKIW7ZiSCC/OAVl1Mpw1gSA24Z4SCFJJfTYq92sJMzM9TXu6oeYALKuPSPsa/vjAllzE3YJDFyR89+1iQ+Hm+EBCQFOhLAg2A3tsmKQKXS4f0R+yYEpgykC4CgaRpo/djLQAEpITfuh3NzamGTnYOLWpAB/8AGGSAKqXYgApH0wJYXYAEhJLsNtfNgJS4pCdC2UajbuxQhLg0/CPpjAUoUqlrJ2ABHuA7E0gTpIa/24qRMmpllIFlbtf3vAwMtkitT1Pon/SPIZQdpqz1pTbwtBwEtOk1TX2T8rQMDKRUAslwdUps5flBwaCqqshmsEpa3sg4GVRQJqtX0SfZpHkMpg61Fgdkt+kSZCcO9BOjdrEgnDzOohwprjvDcOwDenBpzmtJYgBlA7vuoawKarFg97gsDv34eqkunW5qBDnfvxlD5gq17jc7OuAZa1K00bUe+yoKgzq97j64fapKrhr/ADGeHpW+uYt4gOWzbbxgjdTEHKD1f4ldiZSZso31YfZMXLQkKXMpFQ9p5QuZIU5GPWASSACGD7B9hClyWvj1jxIdoTicMkGqcDSA79dCTBxWGFuOh+Tx5ThnI4qXFm3eBisIUvx0k9DHlWG0M5IGpg4nDil5ycwcHZo8qwrPx0MG35wiZLWMiwbAn26dnEmnDzVUuw0qp35xxZYYmUpNrffJvFeHPDFKiqlVSeJoRdoE2Tmsq/8A1RYQlclxZTOX+9EBcmwCVEjfijeEqkrWkUqur1oLEwJkpZahVyA3FHhBXLZJSkn/ALoF4E2TsFNofvRvGBWhSpgTsH79XYXKUtcpaQLK15Dp9mMLSX0zD3+wiBMprZdyDurm+lWkJmyyUlSrgAakfuYTNSE1BZqL8/qgTE0jPZR5qsxv50GclVQqLFtzzf0oM1NKVPoOov8AFCZwpRUvUHcmzsbFUCZa6wlNYa523JqgTUJ881NuSR4gVRgS6ZmaoOObc9yeziLSJhJZh+/QiEzaTaYk1CxrNiDuCreOLRMAE3QjckN8UKmKa0zQ2zKcj4orBBHE5XdXzIVHGJDVl9NVfVCZpQjvVOB5x9zFUFaQtbLVswqJv8UJnJNIqOYWuoC5/NHECVBlr7ujk33PejCF5s3M7dSf1J7WMfg2d6hcP+wMCtySP8XuAaE1oqF9WqdRitQU4QXc+kRDTSQdAWsaoVxGDVAPap39whllKmqLE2ze0aQagpikgsQ92gVM33tifNUzn3wtJzXU9JbXV9hGDJKVkvc7uwO7A9md/Ama6bPz6RQqnKVi93CopUaznYFwCFOx5W1EC6Ug197XNoTvaAVBSXqZ9KVaO1zCEqBAIL30qIZnNzA4h0SagTso9eUFCwVJShbubmqGnJAFKyRoWURbq0JQthlWHd1Mva9i1jGCCwZhJmMQLF2J53btY5uAHSGKhqx/V4QwS4Sip2DAC50L0wlIpUEoTUw2Gu75YSxCaqT/ACi3hlhgKsqanLJ263pgukJypPsAZ7sGTCQkqJpFz3up1bLCSWvTY8hY/DCkprC2DML9fCmFBTOyWYt0cuwyxgmoWKnzPpz9g7M8pEiY6QrmGe0Fctv4EuncUX5WvFaEkEYeWwGyNzqNYUZaWPk0km+iLatzipAAeRKY7BHXxh0BlDDShfZG2+8JWJc0FMmSDoFBNx1d4GMnsFAJ5EU38dYVjZ4SCEpYbUm/zgYyaQkMH/L+t4w8+ZOKgsJAADMGL+89rHCiQKtah7PmIUoNUVgXA2DsPzRfYakdf0XAWSgFw1Vm2O7mqEswLPbRwLbHvQhKpgKpSSWIdm+qBJmgh5KwQD0v8UHD4hiOHM6KIt8lQJGINVcte75WLt0VaPJppPcWLlzZmO/etGFRMlhYUCASGGn7nsz0k4eYWgoTsDcFstgfhhQVSzZnG23wQgppFSdzsbvsRRtBQzdW832sMsFICj0u9L9LimGSbeaw835HLttFIYkCm92HsFqIUAUWAcFrp2ezZYIBWSALAKulnL/ljAUpK2QAGFv/AJKXHaxa+HKCn84Ws3zhWIWCKlpLEOcjGFYgs6VpsTsnePKFKy1jk2RyYOJLJNYFiPMhGJmFQZYFrjKSwudI8tm6ibQCXDlNgdhBxeILATyHc1Oi40eDjJ2qZoFNjdELxU4FuKxYAd2MJOM5MxSluxDANy5p7OJKfJprnboPmY+7LVKCRbdIc6iF2KGUgAh3NNhyEKUkE5k3A9GKkoPeSXAOqYSkzCEICSQNHTdi0HDT1XMq9ncJctaPJcSpLcKk7d0uXYuIGGxA1kh7MclgIGDxANpJKf5RbxjCypktRrQ2UDbbw/s97AdkBu0w+1hyjcMN7/8AKP/Z",
          title:
            "Representation Alignment for Generation: Training Diffusion Transformers Is Easier Than You Think",
          github_repo: "https://github.com/sihyun-yu/REPA",
          framework:
            "https://production-assets.paperswithcode.com/perf/images/frameworks/pytorch-2fbf2cb9.png",
          publish_date: "9 Oct 2024",
          description:
            "Recent studies have shown that the denoising process in (generative) diffusion models can induce meaningful (discriminative) representations inside the model, though the quality of these representations still lags behind those learned through recent self-supervised learning methods.",
          tags: [
            {
              name: "",
              url: "https://paperswithcode.com/sota/image-generation-on-imagenet-256x256",
            },
            {
              name: "",
              url: "https://paperswithcode.com/sota/image-generation-on-imagenet-256x256",
            },
            {
              name: "Denoising",
              url: "https://paperswithcode.com/task/denoising",
            },
            {
              name: "Image Generation",
              url: "https://paperswithcode.com/task/image-generation",
            },
            {
              name: "",
              url: "https://paperswithcode.com/paper/representation-alignment-for-generation#tasks",
            },
          ],
        },
        {
          slug: "/paper/lightrag-simple-and-fast-retrieval-augmented",
          paper_url:
            "https://paperswithcode.com/paper/lightrag-simple-and-fast-retrieval-augmented",
          code_url:
            "https://paperswithcode.com/paper/lightrag-simple-and-fast-retrieval-augmented#code",
          image_url:
            "https://production-media.paperswithcode.com/thumbnails/papergithubrepo/17076b01-dc19-4296-a44d-e2bb9c5b96c6.jpg",
          title: "LightRAG: Simple and Fast Retrieval-Augmented Generation",
          github_repo: "https://github.com/hkuds/lightrag",
          framework: "",
          publish_date: "8 Oct 2024",
          description:
            "Retrieval-Augmented Generation (RAG) systems enhance large language models (LLMs) by integrating external knowledge sources, enabling more accurate and contextually relevant responses tailored to user needs.",
          tags: [
            {
              name: "Information Retrieval",
              url: "https://paperswithcode.com/task/information-retrieval",
            },
            {
              name: "RAG",
              url: "https://paperswithcode.com/task/rag",
            },
            {
              name: "",
              url: "https://paperswithcode.com/paper/lightrag-simple-and-fast-retrieval-augmented#tasks",
            },
          ],
        },
        {
          slug: "/paper/baichuan-omni-technical-report",
          paper_url:
            "https://paperswithcode.com/paper/baichuan-omni-technical-report",
          code_url:
            "https://paperswithcode.com/paper/baichuan-omni-technical-report#code",
          image_url:
            "https://production-media.paperswithcode.com/thumbnails/papergithubrepo/80edf03c-1f28-497a-9f3d-0782ad34a7d0.jpg",
          title: "Baichuan-Omni Technical Report",
          github_repo: "https://github.com/westlake-baichuan-mllm/bc-omni",
          framework: "",
          publish_date: "11 Oct 2024",
          description:
            "The salient multimodal capabilities and interactive experience of GPT-4o highlight its critical role in practical applications, yet it lacks a high-performing open-source counterpart.",
          tags: [
            {
              name: "Language Modelling",
              url: "https://paperswithcode.com/task/language-modelling",
            },
            {
              name: "Large Language Model",
              url: "https://paperswithcode.com/task/large-language-model",
            },
            {
              name: "",
              url: "https://paperswithcode.com/paper/baichuan-omni-technical-report#tasks",
            },
          ],
        },
        {
          slug: "/paper/aria-an-open-multimodal-native-mixture-of",
          paper_url:
            "https://paperswithcode.com/paper/aria-an-open-multimodal-native-mixture-of",
          code_url:
            "https://paperswithcode.com/paper/aria-an-open-multimodal-native-mixture-of#code",
          image_url:
            "data:image/jpeg;base64,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",
          title: "Aria: An Open Multimodal Native Mixture-of-Experts Model",
          github_repo: "https://github.com/rhymes-ai/aria",
          framework:
            "https://production-assets.paperswithcode.com/perf/images/frameworks/pytorch-2fbf2cb9.png",
          publish_date: "8 Oct 2024",
          description: "Information comes in diverse modalities.",
          tags: [
            {
              name: "Instruction Following",
              url: "https://paperswithcode.com/task/instruction-following",
            },
          ],
        },
        {
          slug: "/paper/mle-bench-evaluating-machine-learning-agents",
          paper_url:
            "https://paperswithcode.com/paper/mle-bench-evaluating-machine-learning-agents",
          code_url:
            "https://paperswithcode.com/paper/mle-bench-evaluating-machine-learning-agents#code",
          image_url:
            "data:image/jpeg;base64,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",
          title:
            "MLE-bench: Evaluating Machine Learning Agents on Machine Learning Engineering",
          github_repo: "https://github.com/openai/mle-bench",
          framework: "",
          publish_date: "9 Oct 2024",
          description:
            "We introduce MLE-bench, a benchmark for measuring how well AI agents perform at machine learning engineering.",
          tags: null,
        },
        {
          slug: "/paper/depth-pro-sharp-monocular-metric-depth-in",
          paper_url:
            "https://paperswithcode.com/paper/depth-pro-sharp-monocular-metric-depth-in",
          code_url:
            "https://paperswithcode.com/paper/depth-pro-sharp-monocular-metric-depth-in#code",
          image_url:
            "https://production-media.paperswithcode.com/thumbnails/papergithubrepo/e8996297-ee5a-438f-878b-a62d932f8378.jpg",
          title:
            "Depth Pro: Sharp Monocular Metric Depth in Less Than a Second",
          github_repo: "https://github.com/apple/ml-depth-pro",
          framework:
            "https://production-assets.paperswithcode.com/perf/images/frameworks/pytorch-2fbf2cb9.png",
          publish_date: "2 Oct 2024",
          description:
            "We present a foundation model for zero-shot metric monocular depth estimation.",
          tags: [
            {
              name: "Monocular Depth Estimation",
              url: "https://paperswithcode.com/task/monocular-depth-estimation",
            },
          ],
        },
        {
          slug: "/paper/generalizable-and-animatable-gaussian-head",
          paper_url:
            "https://paperswithcode.com/paper/generalizable-and-animatable-gaussian-head",
          code_url:
            "https://paperswithcode.com/paper/generalizable-and-animatable-gaussian-head#code",
          image_url:
            "https://production-media.paperswithcode.com/thumbnails/papergithubrepo/ca6d0ca1-c624-4890-8665-c934bc7005bd.jpg",
          title: "Generalizable and Animatable Gaussian Head Avatar",
          github_repo: "https://github.com/xg-chu/gagavatar",
          framework:
            "https://production-assets.paperswithcode.com/perf/images/frameworks/pytorch-2fbf2cb9.png",
          publish_date: "10 Oct 2024",
          description:
            "In this paper, we propose Generalizable and Animatable Gaussian head Avatar (GAGAvatar) for one-shot animatable head avatar reconstruction.",
          tags: null,
        },
      ];
    }
  });
</script>

<svelte:head>
  <title>Papers with Code</title>
</svelte:head>

<main id="container--papers">
  <h1 class="margin-y">
    <i class="fa-solid fa-file-contract"></i> Papers with Code •
    <a href="/">home</a>
  </h1>

  {#if loading}
    <p class="section--page-text-center">Loading papers...</p>
  {:else if error}
    <p class="section--page-text-center error-message">{error}</p>
  {:else}
    <div class="papers-grid">
      {#each papers as paper}
        <div class="card--project">
          <img src={paper.image_url} alt={paper.title} class="paper-image" />
          <div class="paper-content">
            <h2>
              <a href={"/papers" + paper.slug} class="paper-title">
                {paper.title}
              </a>
            </h2>
            <div class="paper-description">
              <small>{paper.description}</small>
            </div>

            <div class="paper-links">
              {#if paper.github_repo}
                <a
                  href={paper.github_repo}
                  target="_blank"
                  rel="noopener noreferrer"
                >
                  <i class="fa-brands fa-github"></i> code
                </a>
              {/if}
            </div>

            {#if paper.tags && paper.tags.length > 0}
              <div class="paper-tags">
                {#each paper.tags as tag}
                  {#if tag.name.length != 0}
                    <a
                      href={tag.url}
                      target="_blank"
                      rel="noopener noreferrer"
                      class="tag"
                    >
                      {tag.name}
                    </a>
                  {/if}
                {/each}
              </div>
            {/if}
          </div>
        </div>
      {/each}
    </div>
  {/if}
</main>

<style>
  @import url("https://fonts.googleapis.com/css2?family=Inter:ital,opsz,wght@0,14..32,100..900;1,14..32,100..900&display=swap");

  small {
    opacity: 0.64;
    font-size: small;
    font-style: italic;
  }

  h2 {
    margin-top: 0.5em;
    margin-bottom: 0.75em;
  }

  .paper-title {
    color: var(--mainTextColor-light);
    font-family: "Inter", sans-serif;
    font-optical-sizing: auto;
    font-weight: 400;
    font-style: normal;
  }

  .paper-description {
    margin-bottom: 0.5em;
  }

  .papers-grid {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(300px, 1fr));
    gap: 2em;
  }

  .card--project {
    background-color: var(--mainBgColor);
    overflow: hidden;
    transition: transform 0.3s ease;
  }

  .paper-image {
    width: 100%;
    height: 200px;
    object-fit: cover;
    border-radius: 4px;
  }

  .paper-content {
    padding-top: 0em;
    padding-left: 0.25em;
    padding-right: 0.25em;
    padding-bottom: 0.25em;
  }

  .paper-tags {
    display: flex;
    flex-direction: column;
    flex-wrap: wrap;
  }

  .tag {
    text-transform: lowercase;
    color: var(--mainTextColor);
    padding-top: 0.2em;
    padding-bottom: 0.2em;
    font-size: 0.7em;
  }

  .paper-links {
    display: flex;
    margin-bottom: 0.75em;
    justify-content: flex-start;
  }

  .paper-links a {
    color: var(--mainLinkColor);
    font-size: 1rem;
  }

  .error-message {
    color: #f63737;
  }

  @media (max-width: 600px) {
    .papers-grid {
      grid-template-columns: 1fr;
    }

    .paper-links {
      flex-direction: column;
      gap: 1em;
    }
  }

  .error-message {
    color: #f63737;
    font-weight: bold;
  }
</style>
