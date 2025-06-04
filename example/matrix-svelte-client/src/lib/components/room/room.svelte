<script lang="ts">
  import { ScrollArea } from "$lib/components/ui/scroll-area";
  import { Button } from "$lib/components/ui/button";
  import { Input } from "$lib/components/ui/input";
  import { SendIcon, LoaderIcon } from "@lucide/svelte";
  import { fade } from "svelte/transition";
  import {
    createMatrixRequest,
    submitAsyncRequest,
    type RoomStore,
  } from "tauri-plugin-matrix-svelte-api";
  import { tick } from "svelte";
  import Item from "./items/item.svelte";

  // Pagination settings
  const PAGE_SIZE = 10;
  let isLoadingMore = $state(false);
  let page = $state(1);

  type Props = {
    roomStore: RoomStore;
  };

  let { roomStore }: Props = $props();

  // Messages state
  let newMessage: string = $state("");
  let viewport: HTMLDivElement | null = $state(null);
  let prevScrollHeight = 0;

  // Load more messages when scrolling up
  const loadMoreMessages = async () => {
    if (isLoadingMore || !roomStore.state.tlState?.fullyPaginated) return;

    isLoadingMore = true;
    prevScrollHeight = viewport?.scrollHeight || 0;
    console.log("Loading more messages !");

    try {
      // const olderMessages = await fetchMessages(page + 1);
      // Check if we have more messages
      // if (olderMessages.length < PAGE_SIZE) {
      //   hasMoreMessages = false;
      // }
      // page++;
      // messages = [...olderMessages, ...messages]; // newer messages passed through a dedicated field by the backend ?
      // Maintain scroll position after loading more messages
      setTimeout(() => {
        if (viewport) {
          const newScrollHeight = viewport.scrollHeight;
          viewport.scrollTop = newScrollHeight - prevScrollHeight;
        }
      }, 100);
    } finally {
      isLoadingMore = false;
    }
  };

  // Handle scroll to check when we need to load more messages
  const handleScroll = async (e: Event) => {
    const target = e.target as HTMLDivElement;
    if (target.scrollTop < 100) {
      // Load more when near top
      console.log("Almost reached the top !");
      await loadMoreMessages();
    }
  };

  // Handle sending new message
  const handleSendMessage = async () => {
    if (!newMessage.trim()) return;

    let request = createMatrixRequest.sendTextMessage(
      roomStore.id,
      newMessage,
      {}, // TODO: handle replies and other stuff
    );
    await submitAsyncRequest(request);
    newMessage = "";
  };

  // Handle enter key press
  const handleKeyDown = (e: KeyboardEvent) => {
    if (e.key === "Enter" && !e.shiftKey) {
      e.preventDefault();
      handleSendMessage();
    }
  };

  $effect.pre(() => {
    if (!viewport) return; // not yet mounted

    // reference `messages` array length so that this code re-runs whenever it changes
    roomStore.state.tlState?.items.length;

    // autoscroll when new messages are added
    if (
      viewport.offsetHeight + viewport.scrollTop >
      viewport.scrollHeight - 20
    ) {
      tick().then(() => {
        viewport?.scrollTo(0, viewport.scrollHeight);
      });
    }
  });
</script>

<div class="flex h-[600px] flex-col rounded-lg border bg-background">
  <!-- Chat messages -->
  <ScrollArea bind:viewport onscroll={handleScroll} class="flex-1 p-4">
    <div class="flex flex-col gap-4">
      <!-- Loading indicator -->
      {#if isLoadingMore}
        <div class="flex justify-center py-2" transition:fade|local>
          <LoaderIcon class="h-6 w-6 animate-spin text-muted-foreground" />
        </div>
      {:else if !roomStore.state.tlState?.fullyPaginated}
        <p class="text-center text-sm text-muted-foreground">
          No more messages
        </p>
      {/if}

      <!-- Messages list -->
      {#if roomStore.state.tlState?.items}
        {#each roomStore.state.tlState?.items as item (item.eventId ?? crypto.randomUUID())}
          <div transition:fade|local>
            <Item {item} />
          </div>
        {/each}
      {:else}
        <b>Error: timeline state should be defined</b>
      {/if}
    </div>
  </ScrollArea>

  <!-- Message input -->
  <div class="border-t p-4">
    <div class="flex gap-2">
      <Input
        bind:value={newMessage}
        onkeydown={handleKeyDown}
        placeholder="Type a message..."
        class="flex-1"
      />
      <Button onclick={handleSendMessage} disabled={!newMessage.trim()}>
        <SendIcon class="h-4 w-4" />
        <span class="sr-only">Send message</span>
      </Button>
    </div>
  </div>
</div>
