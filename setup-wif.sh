#!/bin/bash
# ═══════════════════════════════════════════════════════════════
# WIF Setup Script for yt_down CI/CD
# Run this in Google Cloud Shell (https://shell.cloud.google.com)
# or any terminal with gcloud authenticated.
# ═══════════════════════════════════════════════════════════════

set -euo pipefail

# ── Configuration ──────────────────────────────────────────────
GITHUB_REPO="nathanzilgo/rust_yt_down_audio"  # Your GitHub org/repo
POOL_NAME="github-pool"
PROVIDER_NAME="github-provider"
SA_NAME="github-actions-deployer"
SA_DISPLAY="GitHub Actions Deployer"
REGION="us-central1"
AR_REPO_NAME="yt-down"

# ── Auto-detect project ───────────────────────────────────────
PROJECT_ID=$(gcloud config get-value project 2>/dev/null)
if [ -z "$PROJECT_ID" ]; then
    echo "❌ No GCP project configured. Run: gcloud config set project YOUR_PROJECT_ID"
    exit 1
fi
PROJECT_NUMBER=$(gcloud projects describe "$PROJECT_ID" --format='value(projectNumber)')
SA_EMAIL="${SA_NAME}@${PROJECT_ID}.iam.gserviceaccount.com"

echo ""
echo "═══════════════════════════════════════════════════"
echo "  WIF Setup for: ${GITHUB_REPO}"
echo "  GCP Project:   ${PROJECT_ID} (${PROJECT_NUMBER})"
echo "═══════════════════════════════════════════════════"
echo ""

# ── Step 1: Enable APIs ───────────────────────────────────────
echo "📦 Step 1: Enabling required APIs..."
gcloud services enable \
    artifactregistry.googleapis.com \
    run.googleapis.com \
    iamcredentials.googleapis.com \
    iam.googleapis.com \
    secretmanager.googleapis.com \
    --project="$PROJECT_ID" --quiet

echo "   ✅ APIs enabled"

# ── Step 2: Create Artifact Registry repo ─────────────────────
echo ""
echo "📦 Step 2: Creating Artifact Registry repository..."
if gcloud artifacts repositories describe "$AR_REPO_NAME" \
    --location="$REGION" --project="$PROJECT_ID" &>/dev/null; then
    echo "   ⏭️  Repository already exists, skipping"
else
    gcloud artifacts repositories create "$AR_REPO_NAME" \
        --repository-format=docker \
        --location="$REGION" \
        --description="yt_down Docker images" \
        --project="$PROJECT_ID" --quiet
    echo "   ✅ Repository created"
fi

# ── Step 3: Create Service Account ────────────────────────────
echo ""
echo "🔑 Step 3: Creating service account..."
if gcloud iam service-accounts describe "$SA_EMAIL" --project="$PROJECT_ID" &>/dev/null; then
    echo "   ⏭️  Service account already exists, skipping"
else
    gcloud iam service-accounts create "$SA_NAME" \
        --display-name="$SA_DISPLAY" \
        --project="$PROJECT_ID" --quiet
    echo "   ✅ Service account created: ${SA_EMAIL}"
fi

# ── Step 4: Grant IAM roles ──────────────────────────────────
echo ""
echo "🔐 Step 4: Granting IAM roles..."
ROLES=(
    "roles/artifactregistry.writer"
    "roles/run.admin"
    "roles/secretmanager.secretAccessor"
)
for ROLE in "${ROLES[@]}"; do
    gcloud projects add-iam-policy-binding "$PROJECT_ID" \
        --member="serviceAccount:${SA_EMAIL}" \
        --role="$ROLE" \
        --quiet --no-user-output-enabled 2>/dev/null
    echo "   ✅ Granted: ${ROLE}"
done

# Allow SA to act as itself (required for Cloud Run deploys)
gcloud iam service-accounts add-iam-policy-binding "$SA_EMAIL" \
    --member="serviceAccount:${SA_EMAIL}" \
    --role="roles/iam.serviceAccountUser" \
    --project="$PROJECT_ID" \
    --quiet --no-user-output-enabled 2>/dev/null
echo "   ✅ Granted: roles/iam.serviceAccountUser (self)"

# ── Step 5: Create Workload Identity Pool ────────────────────
echo ""
echo "🌐 Step 5: Creating Workload Identity Pool..."
if gcloud iam workload-identity-pools describe "$POOL_NAME" \
    --location="global" --project="$PROJECT_ID" &>/dev/null; then
    echo "   ⏭️  Pool already exists, skipping"
else
    gcloud iam workload-identity-pools create "$POOL_NAME" \
        --location="global" \
        --display-name="GitHub Actions Pool" \
        --project="$PROJECT_ID" --quiet
    echo "   ✅ Pool created"
fi

# ── Step 6: Create OIDC Provider ─────────────────────────────
echo ""
echo "🔗 Step 6: Creating OIDC Provider..."
if gcloud iam workload-identity-pools providers describe "$PROVIDER_NAME" \
    --location="global" \
    --workload-identity-pool="$POOL_NAME" \
    --project="$PROJECT_ID" &>/dev/null; then
    echo "   ⏭️  Provider already exists, skipping"
else
    gcloud iam workload-identity-pools providers create-oidc "$PROVIDER_NAME" \
        --location="global" \
        --workload-identity-pool="$POOL_NAME" \
        --display-name="GitHub Provider" \
        --attribute-mapping="google.subject=assertion.sub,attribute.actor=assertion.actor,attribute.repository=assertion.repository" \
        --attribute-condition="assertion.repository == '${GITHUB_REPO}'" \
        --issuer-uri="https://token.actions.githubusercontent.com" \
        --project="$PROJECT_ID" --quiet
    echo "   ✅ Provider created"
fi

# ── Step 7: Allow WIF to impersonate the SA ──────────────────
echo ""
echo "🔑 Step 7: Binding WIF to service account..."
gcloud iam service-accounts add-iam-policy-binding "$SA_EMAIL" \
    --role="roles/iam.workloadIdentityUser" \
    --member="principalSet://iam.googleapis.com/projects/${PROJECT_NUMBER}/locations/global/workloadIdentityPools/${POOL_NAME}/attribute.repository/${GITHUB_REPO}" \
    --project="$PROJECT_ID" \
    --quiet --no-user-output-enabled 2>/dev/null
echo "   ✅ WIF binding created"

# ── Output: GitHub Secrets ───────────────────────────────────
WIF_PROVIDER="projects/${PROJECT_NUMBER}/locations/global/workloadIdentityPools/${POOL_NAME}/providers/${PROVIDER_NAME}"

echo ""
echo "═══════════════════════════════════════════════════════════"
echo "  ✅ SETUP COMPLETE!"
echo "═══════════════════════════════════════════════════════════"
echo ""
echo "  Add these 3 secrets to GitHub (Settings → Secrets → Actions):"
echo ""
echo "  ┌─────────────────────┬─────────────────────────────────────────────┐"
printf "  │ %-19s │ %-43s │\n" "GCP_PROJECT_ID" "$PROJECT_ID"
echo "  ├─────────────────────┼─────────────────────────────────────────────┤"
printf "  │ %-19s │ %-43s │\n" "WIF_PROVIDER" "$WIF_PROVIDER"
echo "  ├─────────────────────┼─────────────────────────────────────────────┤"
printf "  │ %-19s │ %-43s │\n" "WIF_SERVICE_ACCOUNT" "$SA_EMAIL"
echo "  └─────────────────────┴─────────────────────────────────────────────┘"
echo ""
echo "  After adding secrets, push to master to trigger the first deploy!"
echo ""
