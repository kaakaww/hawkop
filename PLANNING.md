# HawkOp CLI Planning

## Project Overview

HawkOp is a professional-grade Rust CLI companion utility for the StackHawk scanner and platform. It provides developers and security teams with streamlined access to StackHawk's dynamic application security testing (DAST) capabilities directly from the terminal.

This project follows enterprise Rust development standards and includes comprehensive testing, documentation, and CI/CD workflows.

The CLI follows GitHub's `gh` CLI design patterns and supports core StackHawk operations including:
- Authentication and API key management (`init`, `status`)
- Organization and resource management (`org list`, `org set`, `org get`, `org clear`)
- User management (`user list` with role filtering)
- Team management (`team list`)
- Application management (`app list` with status filtering)
- Scan management (`scan list`, `scan get`, `scan alerts`)
- Version information (`version`)
- Self help and contextual help (`help`, `--help`)
- Smart command autocompletion

## Architecture Notes

HawkOp will be a fully functional CLI application with comprehensive StackHawk platform integration. The intent is to replicate and surpass most of the functionality of the StackHawk UI at https://app.stackhawk.com and documented at https://docs.stackhawk.com/web-app/ and its sub-pages. 

The architecture includes:
### Command Structure
- Root command with subcommands following `hawkop <command> <subcommand>` pattern
- Commands include `init`, `version`, `status`, `org`, `app`, `user`, `team`, `scan`, `policy`, `finding`, `report`
- Subcommands include: `list`, `get`, `set`, `clear`, `alerts`
- Consistent flag patterns: `--format`, `--org`, `--limit`, `--app`, `--env`, `--status`, `--severity`

### Configuration Management
- API key storage in `~/.hawkop/config.yaml`
- Default organization and CLI preferences such as default pagination limits also in `~/.hawkop/config.yaml`
- Connection status tracking

### Desired features:
### Implemented Features ✅
[ ] **CLI Framework** - Complete CLI with hierarchical commands largely modeled on GitHub's `gh` CLI
[ ] **Security & Auth** - Secure config storage, JWT management, API key protection
[ ] **Configuration** - `~/.hawkop/config.yaml` with 600 permissions
[ ] **Core Commands** - `init`, `status`, `version`, `org`, `user`, `team`, `app`, `scan`
[ ] **Resource Management** - List and manage orgs, users, teams, applications
[ ] **Scan Analysis** - List scans, view details, analyze security alerts
[ ] **Smart Filtering** - App/env/status/role/severity filters across commands
[ ] **Output Formats** - Professional table formatting + JSON for automation
[ ] **Enterprise Ready** - Pagination, organization awareness, role-based access
[ ] **Real Security Data** - Live integration showing actual vulnerability findings
[ ] **Extensible Architecture** - Clean patterns for adding new commands and reports
[ ] **Production Quality** - Error handling, validation, user-friendly messaging
[ ] **Testing Infrastructure** - Comprehensive test suites with testify framework
[ ] **CI/CD Pipeline** - GitHub Actions for automated testing and releases
[ ] **Release Management** - Multi-platform binary distribution via GitHub Releases
[ ] **Professional Standards** - MIT license, comprehensive docs, contribution guidelines
[ ] **YAML Configuration** - Human-readable YAML format for configuration files

### API Standards Compliance ✅
- **Rate Limiting**: 360 requests/minute compliance with initial unlimited burst backing down to 167ms intervals to balance initial speed and responsiveness with eventual pacing to avoid dropped connections.
- **Pagination**: Default pageSize=1000 (maximum) to minimize API requests
- **Error Handling**: Comprehensive HTTP status code handling (400, 401, 403, 404, 409, 422, 429)
- **Retry Logic**: Automatic JWT refresh on 401 or expiry, rate limit retry on 429
- **Query Parameters**: Proper URL encoding and parameter validation

### Future Enhancement Opportunities
1. **Advanced Scan Features**
    - `scan finding` - Individual finding details with request/response data
    - `scan message` - Raw HTTP request/response analysis
    - Scan filtering by date ranges and criticality

2. **Enterprise Reporting**
    - `app summary` - Cross-application security posture dashboard
    - `app report` - MTTR analysis, scan coverage, policy compliance
    - Historical trending and ROI metrics

3. **Application Deep Dive**
    - `app get` - Application metadata, configuration, policy assignment
    - `app scans` - Scan history and trends for specific applications
    - Attack Surface repository mappings

4. **Policy & Configuration Management**
    - `policy list` - Available scan policies
    - Policy assignment and configuration
    - Environment and configuration management

5. **Advanced Features**
    - Export capabilities (CSV, PDF reports)
    - Interactive mode for guided workflows
    - Scan result comparison and diff analysis

6. **API Helpers**
    - CRUD helper for list endpoints that require full list updates
    - API pagination helper

7. **Automation Helpers**
    - HawkScan install automation helper for all platforms (find version, download, install, set path, check JDK, etc)

8. **Repository/Data Abstraction Layer**
    - Replace simple list caching with full API response payload caching (includes pagination metadata: `hasNext`, `currentPage`, `next` cursor, etc.)
    - Implement Repository pattern to abstract cache vs API:
      ```
      src/repository/
        mod.rs      - Repository trait
        apps.rs     - AppRepository
        orgs.rs     - OrgRepository
        scans.rs    - ScanRepository
      ```
    - Repository trait interface:
      ```rust
      pub trait Repository<T> {
          async fn list(&self, query: Query) -> Result<Page<T>>;
          async fn get(&self, id: &str) -> Result<Option<T>>;
          fn invalidate(&self);
      }
      ```
    - Benefits:
      - Callers don't care where data comes from (cache vs API)
      - Full response payloads cached with metadata
      - Smart cache invalidation
      - Support cursor-based pagination from cache
      - Enable hybrid caching (stale-while-revalidate)

9. **Hybrid Caching Strategy**
    - Show cached data immediately for fast UX
    - Refresh stale data in background
    - Update display if data changed
    - Configurable staleness thresholds per resource type

### Configuration File Format
The config file is stored at `~/.hawkop/config.yaml` with 600 permissions:
```yaml
api_key: your-api-key
org_id: optional-default-org-id
jwt:
  token: jwt-token
  expires_at: 2024-12-25T15:30:45Z
```

## Reference Materials

### StackHawk Resources

- StackHawk API OpenAPI Spec: https://download.stackhawk.com/openapi/stackhawk-openapi.json
- StackHawk API Authentication Reference: https://apidocs.stackhawk.com/reference/login
- StackHawk API Documentation: https://apidocs.stackhawk.com/
- StackHawk Documentation: https://docs.stackhawk.com/
- HawkScan configuration schema: https://download.stackhawk.com/hawk/jsonschema/hawkconfig.json