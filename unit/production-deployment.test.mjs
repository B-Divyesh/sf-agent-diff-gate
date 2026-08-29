import assert from 'node:assert/strict';
import test from 'node:test';

import {
  DATABASE_URL,
  DEPLOYMENT_CONFIG_VERSION,
  assertProductionContract,
  productionContractErrors,
  renderProductionTemplate,
} from '../deploy/production-contract.mjs';

const runtime = {
  PUBLIC_BASE_URL: 'https://agent-diff-gate.sociobot.in',
  ENTRA_AUTHORITY: 'https://sociobotcustomers.ciamlogin.com/tenant',
  ENTRA_TENANT_ID: 'tenant',
  ENTRA_CLIENT_ID: 'client',
  ENTRA_TEAM_CLAIM: 'oid',
};
const image = 'sociobotregistry.azurecr.io/sf-agent-diff-gate:repair-sha';
const storageName = 'agent-diff-gate-data-v4';

function factoryStatelessApp() {
  return {
    properties: {
      configuration: { activeRevisionsMode: 'Single' },
      template: {
        containers: [
          {
            name: 'app',
            image: 'sociobotregistry.azurecr.io/sf-agent-diff-gate:old-sha',
            resources: { cpu: 0.5, memory: '1Gi' },
            env: [
              { name: 'PORT', value: '8080' },
              { name: 'UNCHANGED_SECRET', secretRef: 'github-key' },
            ],
          },
        ],
        scale: { minReplicas: 1, maxReplicas: 3, cooldownPeriod: 300 },
        volumes: null,
      },
    },
  };
}

test('regression: the factory stateless template is rejected', () => {
  const errors = productionContractErrors(factoryStatelessApp(), { image, storageName, runtime });
  assert(errors.includes('SQLite requires exactly one replica'));
  assert(errors.some(error => error.includes('Azure Files volume data')));
  assert(errors.includes('Azure Files volume data must be mounted at /data'));
  assert(errors.includes('DATABASE_URL must match the production contract'));
  assert.throws(
    () => assertProductionContract(factoryStatelessApp(), { image, storageName, runtime }),
    /Unsafe production configuration/,
  );
});

test('regression: one render atomically installs the image and durable SQLite contract', () => {
  const original = factoryStatelessApp();
  const template = renderProductionTemplate(original, { image, storageName, runtime });
  const repaired = {
    properties: {
      configuration: { activeRevisionsMode: 'Single' },
      template,
    },
  };

  assert.doesNotThrow(() => assertProductionContract(repaired, { image, storageName, runtime }));
  assert.deepEqual(template.scale, { minReplicas: 1, maxReplicas: 1, cooldownPeriod: 300 });
  assert.deepEqual(template.volumes, [{ name: 'data', storageType: 'AzureFile', storageName }]);
  assert.deepEqual(template.containers[0].volumeMounts, [{ volumeName: 'data', mountPath: '/data' }]);
  assert.equal(template.containers[0].image, image);
  assert(template.containers[0].env.some(entry => entry.name === 'DATABASE_URL' && entry.value === DATABASE_URL));
  assert(
    template.containers[0].env.some(
      entry => entry.name === 'DEPLOYMENT_CONFIG_VERSION' && entry.value === DEPLOYMENT_CONFIG_VERSION,
    ),
  );
  assert(template.containers[0].env.some(entry => entry.name === 'UNCHANGED_SECRET' && entry.secretRef === 'github-key'));
  assert.deepEqual(original, factoryStatelessApp(), 'rendering must not mutate the Azure response');
});

test('regression: scale, storage, mount, database path, and image are all mandatory', () => {
  const base = factoryStatelessApp();
  const safeTemplate = renderProductionTemplate(base, { image, storageName, runtime });
  const mutations = [
    template => {
      template.scale.maxReplicas = 3;
    },
    template => {
      template.volumes = null;
    },
    template => {
      template.containers[0].volumeMounts = [];
    },
    template => {
      template.containers[0].env.find(entry => entry.name === 'DATABASE_URL').value = 'sqlite:/data/other.db';
    },
    template => {
      template.containers[0].image = 'sociobotregistry.azurecr.io/sf-agent-diff-gate:wrong';
    },
  ];

  for (const mutate of mutations) {
    const template = structuredClone(safeTemplate);
    mutate(template);
    assert.throws(
      () =>
        assertProductionContract(
          { properties: { configuration: { activeRevisionsMode: 'Single' }, template } },
          { image, storageName, runtime },
        ),
      /Unsafe production configuration/,
    );
  }
});
