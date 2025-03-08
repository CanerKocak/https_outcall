# Claude API for Internet Computer Canisters

This document provides instructions on how to use the Claude API endpoint from your Internet Computer canisters.

## Overview

The Claude API endpoint allows your canisters to make calls to Anthropic's Claude AI model. The endpoint handles:

- Authentication
- Request deduplication
- Response caching
- Error handling

## API Endpoint

The Claude API endpoint is available at:

```
https://your-server-url/claude
```

## Request Format

To call the Claude API, your canister should make an HTTP outcall to the endpoint with the following JSON payload:

```json
{
  "canister_id": "your-canister-id",
  "request_id": "unique-request-id",
  "system": "Optional system prompt",
  "messages": [
    {
      "role": "user",
      "content": "Your message to Claude"
    }
  ],
  "max_tokens": 1000,
  "temperature": 0.7
}
```

### Request Fields

- `canister_id`: The principal ID of your canister (required)
- `request_id`: A unique ID for this request to enable deduplication (required)
- `system`: Optional system prompt to set context for Claude
- `messages`: Array of message objects with `role` and `content` (required)
- `max_tokens`: Maximum number of tokens in the response (optional, default: 1000)
- `temperature`: Controls randomness in the response (optional, default: 0.7)

## Response Format

The API returns a JSON response with the following structure:

```json
{
  "success": true,
  "message": "Claude API response",
  "data": {
    "id": "response-id",
    "content": [
      {
        "type": "text",
        "text": "Claude's response text"
      }
    ],
    "model": "claude-3-sonnet-20240229",
    "role": "assistant",
    "stop_reason": "end_turn",
    "stop_sequence": null,
    "usage": {
      "input_tokens": 123,
      "output_tokens": 456
    }
  }
}
```

### Response Fields

- `success`: Boolean indicating if the request was successful
- `message`: A message describing the result
- `data`: The Claude API response data (if successful)
  - `id`: Unique ID for the response
  - `content`: Array of content objects with `type` and `text`
  - `model`: The Claude model used
  - `role`: Always "assistant"
  - `stop_reason`: Reason why the response stopped
  - `stop_sequence`: Stop sequence if any
  - `usage`: Token usage information

## Deduplication

The API implements deduplication based on the `canister_id` and `request_id`. If you send the same request multiple times, the API will return the cached response without calling the Claude API again.

Cached responses expire after 30 minutes.

## Error Handling

If an error occurs, the API will return a JSON response with:

```json
{
  "success": false,
  "message": "Error message",
  "data": null
}
```

## Example Usage in Motoko

```motoko
import Debug "mo:base/Debug";
import Principal "mo:base/Principal";
import Text "mo:base/Text";
import Time "mo:base/Time";
import Int "mo:base/Int";
import Http "mo:base/Http";
import Blob "mo:base/Blob";
import JSON "mo:base/JSON"; // You'll need a JSON library

actor {
  // Define types for Claude API
  type ClaudeMessage = {
    role : Text;
    content : Text;
  };

  type ClaudeRequest = {
    canister_id : Text;
    request_id : Text;
    system : ?Text;
    messages : [ClaudeMessage];
    max_tokens : ?Nat32;
    temperature : ?Float;
  };

  // Call Claude API
  public func askClaude(question : Text) : async Text {
    let canisterId = Principal.toText(Principal.fromActor(this));
    let requestId = "request-" # Int.toText(Time.now());
    
    let request : ClaudeRequest = {
      canister_id = canisterId;
      request_id = requestId;
      system = ?"You are a helpful assistant for the Internet Computer.";
      messages = [
        {
          role = "user";
          content = question;
        }
      ];
      max_tokens = ?1000;
      temperature = ?0.7;
    };
    
    // Convert request to JSON
    let requestJson = /* Convert request to JSON */;
    
    // Make HTTP outcall to Claude API
    let httpRequest : Http.Request = {
      url = "https://your-server-url/claude";
      method = #post;
      body = Blob.fromArray(Text.encodeUtf8(requestJson));
      headers = [
        { name = "Content-Type"; value = "application/json" },
        { name = "X-API-Key"; value = "your-api-key" }
      ];
    };
    
    try {
      let httpResponse = await Http.request(httpRequest);
      
      if (httpResponse.status == 200) {
        // Parse response JSON
        let responseJson = Text.decodeUtf8(Blob.toArray(httpResponse.body));
        
        // Extract Claude's response text
        // This is simplified - you'll need proper JSON parsing
        return "Claude's response"; 
      } else {
        return "Error: HTTP status " # Nat.toText(httpResponse.status);
      }
    } catch (e) {
      return "Error making HTTP request: " # Debug.trap(e);
    }
  };
}
```

## Notes

- The API key is currently not validated, but you should include one in the `X-API-Key` header for future compatibility.
- The Claude API has rate limits and token limits. Be mindful of these when making requests.
- Responses are cached for 30 minutes to reduce API calls and costs.
- For production use, consider implementing additional security measures. 