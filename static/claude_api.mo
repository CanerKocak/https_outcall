// Claude API Interface for Internet Computer canisters

// Message structure for Claude API
type ClaudeMessage = {
  role : Text;
  content : Text;
};

// Request structure for Claude API
type ClaudeRequest = {
  canister_id : Text;
  request_id : Text;
  system : ?Text;
  messages : [ClaudeMessage];
  max_tokens : ?Nat32;
  temperature : ?Float;
};

// Content structure for Claude API response
type ClaudeContent = {
  #type : Text;
  text : Text;
};

// Usage structure for Claude API response
type ClaudeUsage = {
  input_tokens : Nat32;
  output_tokens : Nat32;
};

// Response structure for Claude API
type ClaudeResponse = {
  id : Text;
  content : [ClaudeContent];
  model : Text;
  role : Text;
  stop_reason : ?Text;
  stop_sequence : ?Text;
  usage : ?ClaudeUsage;
};

// API response wrapper
type ApiResponse<T> = {
  success : Bool;
  message : Text;
  data : ?T;
};

// Claude API actor interface
actor {
  // Call Claude API
  public func callClaude(request : ClaudeRequest) : async ApiResponse<ClaudeResponse>;
};

// Example usage:
/*
import Claude "canister:claude_api";

actor {
  public func generateText() : async Text {
    let request : ClaudeRequest = {
      canister_id = Principal.toText(Principal.fromActor(this));
      request_id = "unique-request-id-" # Int.toText(Time.now());
      system = ?"You are a helpful assistant for the Internet Computer.";
      messages = [
        {
          role = "user";
          content = "Hello, Claude! What can you tell me about the Internet Computer Protocol?";
        }
      ];
      max_tokens = ?500;
      temperature = ?0.7;
    };
    
    let response = await Claude.callClaude(request);
    
    if (response.success) {
      switch (response.data) {
        case (null) { "No response data" };
        case (?data) {
          if (data.content.size() > 0) {
            data.content[0].text;
          } else {
            "Empty response";
          }
        };
      };
    } else {
      "Error: " # response.message;
    };
  };
};
*/ 