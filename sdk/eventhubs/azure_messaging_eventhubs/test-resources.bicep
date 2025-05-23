@description('Base name of the resources to be created')
param baseName string = resourceGroup().name

@description('Azure region for the resources')
param location string = resourceGroup().location

@description('Storage endpoint suffix')
param StorageEndpointSuffix string = 'core.windows.net'

// Variables for unique naming
//var uniqueSuffix = toLower(uniqueString(resourceGroup().id))
var storageAccountName = 'blb${baseName}'
var eventHubsNamespaceName = 'eh-${baseName}'
var eventHubName = 'testEventHub'
var eventHubConsumerGroupName = '$Default'

// Storage Account
resource storage 'Microsoft.Storage/storageAccounts@2021-09-01' = {
  name: storageAccountName
  location: location
  sku: {
    name: 'Standard_LRS'
  }
  kind: 'StorageV2'
  properties: {
    accessTier: 'Hot'
    supportsHttpsTrafficOnly: true
    minimumTlsVersion: 'TLS1_2'
    allowSharedKeyAccess: false
  }
}

// Event Hubs Namespace
resource eventHubsNamespace 'Microsoft.EventHub/namespaces@2021-11-01' = {
  name: eventHubsNamespaceName
  location: location
  sku: {
    name: 'Standard'
    tier: 'Standard'
    capacity: 1
  }
  properties: {
    isAutoInflateEnabled: false
    disableLocalAuth: true
    maximumThroughputUnits: 0
  }
}

// Event Hub
resource eventHub 'Microsoft.EventHub/namespaces/eventhubs@2021-11-01' = {
  parent: eventHubsNamespace
  name: eventHubName
  properties: {
    messageRetentionInDays: 7
    partitionCount: 4
  }
}

// Event Hub Consumer Group
resource consumerGroup 'Microsoft.EventHub/namespaces/eventhubs/consumergroups@2021-11-01' = {
  parent: eventHub
  name: eventHubConsumerGroupName
}

// Create an authorization rule for the Event Hub
resource eventHubAuthRule 'Microsoft.EventHub/namespaces/eventhubs/authorizationRules@2021-11-01' = {
  parent: eventHub
  name: 'SendListenAccessRule'
  properties: {
    rights: [
      'Send'
      'Listen'
    ]
  }
}

// Create a container in the storage account for checkpoint store
resource blobService 'Microsoft.Storage/storageAccounts/blobServices@2021-09-01' = {
  parent: storage
  name: 'default'
}

resource container 'Microsoft.Storage/storageAccounts/blobServices/containers@2021-09-01' = {
  parent: blobService
  name: 'checkpoints'
  properties: {
    publicAccess: 'None'
  }
}

// Outputs
output EVENTHUB_NAME string = eventHub.name
output EVENTHUBS_NAMESPACE string = eventHubsNamespace.name
output EVENTHUBS_HOST string = '${eventHubsNamespace.name}.servicebus.windows.net'
//output EVENTHUBS_CONNECTION_STRING string = listKeys(eventHubAuthRule.id, eventHubAuthRule.apiVersion).primaryConnectionString
output AZURE_STORAGE_ACCOUNT_NAME string = storage.name
//output AZURE_STORAGE_ACCOUNT_KEY string = listKeys(storage.id, storage.apiVersion).keys[0].value
output STORAGE_ENDPOINT_SUFFIX string = StorageEndpointSuffix
output AZURE_STORAGE_BLOB_ENDPOINT string = storage.properties.primaryEndpoints.blob
output RESOURCE_GROUP string = resourceGroup().name
