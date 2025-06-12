use std::path::{Path, PathBuf};
use tokio::fs;
use eyre::Result;

/// Recursively discovers all directories that contain .tf files, starting from the given directory.
/// 
/// # Arguments
/// * `dir_path` - The root directory path to start searching from
/// 
/// # Returns
/// A vector of directory paths that contain .tf files, including the root directory if it contains .tf files
/// 
/// # Errors
/// Returns an error if there are filesystem access issues or if the provided path is not a directory
pub async fn discover_terraform_source_dirs<P: AsRef<Path>>(dir_path: P) -> Result<Vec<PathBuf>> {
    let dir_path = dir_path.as_ref();
    let mut terraform_dirs = Vec::new();
    let mut dirs_to_process = vec![dir_path.to_path_buf()];
    
    while let Some(current_dir) = dirs_to_process.pop() {
        // Check if the current directory is accessible
        let mut entries = match fs::read_dir(&current_dir).await {
            Ok(entries) => entries,
            Err(_) => continue, // Skip directories we can't read
        };
        
        let mut has_tf_files = false;
        
        // Check for .tf files and collect subdirectories
        while let Some(entry) = entries.next_entry().await? {
            let path = entry.path();
            let metadata = entry.metadata().await?;
            
            if metadata.is_file() {
                if let Some(extension) = path.extension() {
                    if extension == "tf" {
                        has_tf_files = true;
                    }
                }
            } else if metadata.is_dir() {
                dirs_to_process.push(path);
            }
        }
        
        // If current directory has .tf files, add it to the result
        if has_tf_files {
            terraform_dirs.push(current_dir);
        }
    }
    
    Ok(terraform_dirs)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use tokio::fs;
    
    #[tokio::test]
    async fn test_discover_terraform_source_dirs() -> Result<()> {
        // Create a temporary directory structure for testing
        let temp_dir = TempDir::new()?;
        let temp_path = temp_dir.path();
        
        // Create directory structure:
        // temp/
        // ├── main.tf
        // ├── subdir1/
        // │   ├── resource.tf
        // │   └── nested/
        // │       └── provider.tf
        // └── subdir2/
        //     └── README.md (no .tf files)
        
        // Create main.tf in root
        fs::write(temp_path.join("main.tf"), "# Main terraform file").await?;
        
        // Create subdir1 with .tf file
        let subdir1 = temp_path.join("subdir1");
        fs::create_dir(&subdir1).await?;
        fs::write(subdir1.join("resource.tf"), "# Resource terraform file").await?;
        
        // Create nested directory in subdir1 with .tf file
        let nested = subdir1.join("nested");
        fs::create_dir(&nested).await?;
        fs::write(nested.join("provider.tf"), "# Provider terraform file").await?;
        
        // Create subdir2 without .tf files
        let subdir2 = temp_path.join("subdir2");
        fs::create_dir(&subdir2).await?;
        fs::write(subdir2.join("README.md"), "# No terraform files here").await?;
        
        // Test the function
        let result = discover_terraform_source_dirs(temp_path).await?;
        
        // Verify results
        assert_eq!(result.len(), 3);
        assert!(result.contains(&temp_path.to_path_buf()));
        assert!(result.contains(&subdir1));
        assert!(result.contains(&nested));
        assert!(!result.contains(&subdir2));
        
        Ok(())
    }
    
    #[tokio::test]
    async fn test_empty_directory() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let temp_path = temp_dir.path();
        
        let result = discover_terraform_source_dirs(temp_path).await?;
        assert!(result.is_empty());
        
        Ok(())
    }
    
    #[tokio::test]
    async fn test_directory_with_only_subdirs_no_tf() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let temp_path = temp_dir.path();
        
        // Create subdirectory without .tf files
        let subdir = temp_path.join("subdir");
        fs::create_dir(&subdir).await?;
        fs::write(subdir.join("config.json"), "{}").await?;
        
        let result = discover_terraform_source_dirs(temp_path).await?;
        assert!(result.is_empty());
        
        Ok(())
    }
}