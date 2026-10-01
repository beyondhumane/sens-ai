<script>
  let count = 0;

  function increment() {
    count += 1;
  }

  function forgotten() {
    return count;
  }
</script>

<button on:click={increment}>{count}</button>
