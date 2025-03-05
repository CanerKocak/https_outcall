/**
 * Utility functions for interacting with the canister registration API
 */

// The API URL for canister registration
const CANISTER_API_URL = 'http://134.209.193.115:8080';

/**
 * Register a canister with the API
 * @param principal The principal ID of the user who created the canister
 * @param canisterId The canister ID to register
 * @param canisterType The type of canister ('token_backend' or 'miner')
 * @returns A promise that resolves to the API response
 */
export async function registerCanister(
  principal: string,
  canisterId: string,
  canisterType: 'token_backend' | 'miner'
): Promise<any> {
  try {
    // Map token_backend to token for the API
    const apiCanisterType = canisterType === 'token_backend' ? 'token' : canisterType;
    
    const response = await fetch(`${CANISTER_API_URL}/canisters`, {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
      },
      body: JSON.stringify({
        principal,
        canister_id: canisterId,
        canister_type: apiCanisterType,
        module_hash: null
      })
    });

    if (!response.ok) {
      const errorText = await response.text();
      console.error(`Error registering canister: ${errorText}`);
      return { success: false, error: errorText };
    }

    const data = await response.json();
    return { success: true, data };
  } catch (error) {
    console.error('Error registering canister:', error);
    return { success: false, error: error instanceof Error ? error.message : 'Unknown error' };
  }
} 