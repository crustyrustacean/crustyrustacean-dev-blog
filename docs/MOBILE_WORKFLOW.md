# Mobile Blogging Workflow Guide

This guide explains how to post articles to your blog from your iPhone using the Craft app and iOS Shortcuts.

## Overview

The mobile workflow allows you to:
- Write blog posts in the Craft app on your iPhone
- Publish them directly to your blog with a single tap
- Use markdown formatting
- Add tags and metadata
- No computer needed!

## Setup Steps

### 1. Generate an API Key

1. Log in to your blog admin dashboard
2. Go to **Admin** → **API Keys**
3. Click "Generate New Key"
4. Give it a name (e.g., "iPhone Shortcut")
5. Copy the generated API key (you won't be able to see it again!)

### 2. Create the iOS Shortcut

#### Option A: Manual Setup

1. Open the **Shortcuts** app on your iPhone
2. Tap **+** to create a new shortcut
3. **Important:** Tap the settings icon (ⓘ) at the bottom and configure:
   - Toggle ON "Show in Share Sheet"
   - Under "Share Sheet Types" select "Text"
   - This allows the shortcut to receive text from Craft
4. Add the following actions:

   **Action 1: Receive "Shortcut Input" from Share Sheet**
   - Search for "Shortcut Input" action
   - This receives the text from Craft when you share

   **Action 2: Set Variable**
   - Name: `Body`
   - Value: Shortcut Input (from previous action)

   **Action 3: Ask for Input**
   - Prompt: "What's the title of your post?"
   - Input type: Text

   **Action 4: Set Variable**
   - Name: `Title`
   - Value: (Output from previous action)

   **Action 5: Get Contents of URL**
   - URL: `https://your-blog.com/api/mobile/upload`
   - Method: `POST`
   - Headers:
     - `X-API-Key`: `[Your API Key from step 1]`
     - `Content-Type`: `application/json`
   - Request Body: `JSON`
   - JSON content:
     ```json
     {
       "title": "[Title variable]",
       "body": "[Body variable]"
     }
     ```

   **Action 6: Show Result**
   - This will display the response from the server

4. Name your shortcut (e.g., "Post to Blog")
5. Tap **Done** to save

#### Option B: Quick Setup JSON

If you prefer, you can import a shortcut using this JSON configuration:

```json
{
  "WFWorkflowActions": [
    {
      "WFWorkflowActionIdentifier": "is.workflow.actions.gettext",
      "WFWorkflowActionParameters": {
        "WFTextActionText": {
          "WFSerializationType": "WFTextTokenAttachment",
          "Value": {
            "attachmentsByRange": {
              "0,1": {
                "Type": "ShortcutInput"
              }
            },
            "string": " "
          }
        }
      }
    }
  ]
}
```

### 3. Configure Craft App

1. Open the **Craft** app
2. Write your blog post using markdown
3. When ready to publish:
   - Tap the **Share** button
   - Select **Shortcuts**
   - Choose your "Post to Blog" shortcut
   - Enter the title when prompted
   - Wait for confirmation!

## Usage Tips

### Writing in Markdown

Your blog supports full markdown syntax:

```markdown
# Heading 1
## Heading 2

**Bold text**
*Italic text*

- Bullet point
- Another point

[Link text](https://example.com)

```code blocks```
```

### Adding Tags

To add tags to your post, modify the shortcut to include a tags field:

```json
{
  "title": "[Title variable]",
  "body": "[Body variable]",
  "tags": ["rust", "mobile", "blogging"]
}
```

You can add an "Ask for Input" action to dynamically specify tags.

### Adding a Description

By default, the first 200 characters of your post will be used as the description. To provide a custom description:

```json
{
  "title": "[Title variable]",
  "body": "[Body variable]",
  "description": "[Description variable]"
}
```

## API Endpoint Reference

### POST /api/mobile/upload

Upload a new article using an API key.

**Headers:**
- `X-API-Key`: Your API key
- `Content-Type`: application/json

**Request Body:**
```json
{
  "title": "My Blog Post",
  "body": "# Markdown content here\n\nYour post content...",
  "description": "Optional custom description",
  "tags": ["optional", "array", "of", "tags"]
}
```

**Response:**
```json
{
  "success": true,
  "message": "Article created successfully",
  "slug": "my-blog-post",
  "url": "/articles/my-blog-post",
  "id": "uuid-here"
}
```

## Troubleshooting

### "Invalid API Key" Error

- Make sure you copied the entire API key
- Check that there are no extra spaces
- Verify the `X-API-Key` header is set correctly

### "Missing title field" Error

- Ensure the title input is being captured
- Check that the title variable is properly referenced in the JSON

### "Failed to connect" Error

- Check your internet connection
- Verify the blog URL is correct
- Make sure the blog is running and accessible

### Article Not Appearing

- Check the blog's admin dashboard
- Verify you're logged in as the correct user
- The article slug might differ from the title (check the response)

## Security Best Practices

1. **Keep your API keys private** - Don't share them with anyone
2. **Delete unused keys** - Remove API keys you're no longer using
3. **Use descriptive names** - Name your keys by device/app for easy tracking
4. **Monitor usage** - Check the "Last Used" column to spot unusual activity

## Advanced: Craft Templates

You can create Craft templates for different types of blog posts:

**Technical Tutorial Template:**
```markdown
# [Title]

## Overview
[Brief introduction]

## Prerequisites
- Item 1
- Item 2

## Step-by-Step Guide

### Step 1: [Title]
[Content]

### Step 2: [Title]
[Content]

## Conclusion
[Summary]

tags: rust, tutorial, programming
```

Save these as templates in Craft and use them as starting points for your posts.

## Questions?

- Check the blog admin dashboard at `/admin/api-keys`
- Review the API key management interface for usage statistics
- Look at recent posts to see the markdown rendering

Happy mobile blogging! 📱✍️
