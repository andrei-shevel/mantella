<script lang="ts" module>
  import ArrowLeft from "@lucide/svelte/icons/arrow-left";
  import ArrowRight from "@lucide/svelte/icons/arrow-right";
  import Bookmark from "@lucide/svelte/icons/bookmark";
  import BookmarkPlus from "@lucide/svelte/icons/bookmark-plus";
  import File from "@lucide/svelte/icons/file";
  import Folder from "@lucide/svelte/icons/folder";
  import Minus from "@lucide/svelte/icons/minus";
  import MoveHorizontal from "@lucide/svelte/icons/move-horizontal";
  import PanelLeft from "@lucide/svelte/icons/panel-left";
  import Pin from "@lucide/svelte/icons/pin";
  import Plus from "@lucide/svelte/icons/plus";
  import Search from "@lucide/svelte/icons/search";
  import X from "@lucide/svelte/icons/x";

  const icons = {
    "arrow-left": ArrowLeft,
    "arrow-right": ArrowRight,
    bookmark: Bookmark,
    "bookmark-plus": BookmarkPlus,
    file: File,
    "fit-width": MoveHorizontal,
    folder: Folder,
    minus: Minus,
    "panel-left": PanelLeft,
    pin: Pin,
    plus: Plus,
    search: Search,
    x: X,
  };

  export type IconName = keyof typeof icons;
</script>

<script lang="ts">
  let {
    name,
    size = 16,
    filled = false,
  }: { name: IconName; size?: number; filled?: boolean } = $props();

  const LucideIcon = $derived(icons[name]);
</script>

<LucideIcon
  {size}
  strokeWidth={1.7}
  fill={filled ? "currentColor" : "none"}
/>
