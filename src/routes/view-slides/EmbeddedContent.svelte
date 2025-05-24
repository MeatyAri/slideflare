<script>
	let { content } = $props();
</script>

<!DOCTYPE html>
<html lang="en">
<head>
<meta charset="UTF-8">
<meta name="viewport" content="width=device-width, initial-scale=1.0">
<script src="../../tailwind-min.js"></script>
<!-- <script>
    // Optional: Configure Tailwind
    tailwind.config = {
        theme: {
            extend: {
                // Your custom theme extensions
            }
        }
    }
</script> -->
</head>
<body class="p-6 bg-white">
<div class="prose prose-lg max-w-none">
    {@html content}
</div>
</body>
</html>
